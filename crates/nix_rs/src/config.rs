//! Rust module for `nix show-config`

use std::{convert::Infallible, str::FromStr};

use serde::{Deserialize, Serialize};
use serde_with::DeserializeFromStr;
use tracing::instrument;
use url::Url;

use super::flake::system::System;

/// Nix configuration spit out by `nix show-config`
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct NixConfig {
    pub cores: ConfigVal<i32>,
    pub experimental_features: ConfigVal<Vec<String>>,
    pub extra_platforms: ConfigVal<Vec<String>>,
    pub flake_registry: ConfigVal<String>,
    pub max_jobs: ConfigVal<i32>,
    pub substituters: ConfigVal<Vec<Url>>,
    pub system: ConfigVal<System>,
    pub trusted_users: ConfigVal<Vec<TrustedUserValue>>,
}

/// The value for each 'nix show-config --json' key.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfigVal<T> {
    /// Current value in use.
    pub value: T,
    /// Default value by Nix.
    pub default_value: T,
    /// Description of this config item.
    pub description: String,
}

impl NixConfig {
    /// Get the output of `nix show-config`

    #[instrument(name = "show-config")]
    pub async fn from_nix(
        nix_cmd: &super::command::NixCmd,
    ) -> Result<NixConfig, super::command::NixCmdError> {
        let v = nix_cmd
            .run_with_args_expecting_json(&["show-config", "--json"])
            .await?;
        Ok(v)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, DeserializeFromStr)]
pub enum TrustedUserValue {
    /// All users are trusted
    All,
    /// A specific user is trusted
    User(String),
    /// Users belonging to a specific group are trusted
    Group(String),
}

impl TrustedUserValue {
    fn from_str(s: &str) -> Self {
        // In nix.conf, groups are prefixed with '@'. '*' means all users are
        // trusted.
        if s == "*" {
            return Self::All;
        }
        match s.strip_prefix('@') {
            Some(s) => Self::Group(s.to_string()),
            None => Self::User(s.to_string()),
        }
    }

    pub fn display_original(val: &[TrustedUserValue]) -> String {
        val.iter()
            .map(|x| match x {
                TrustedUserValue::All => "*".to_string(),
                TrustedUserValue::User(x) => x.to_string(),
                TrustedUserValue::Group(x) => format!("@{}", x),
            })
            .collect::<Vec<String>>()
            .join(" ")
    }
}

impl From<String> for TrustedUserValue {
    fn from(s: String) -> Self {
        Self::from_str(&s)
    }
}

impl FromStr for TrustedUserValue {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::from_str(s))
    }
}

#[tokio::test]
async fn test_nix_config() -> Result<(), crate::command::NixCmdError> {
    let v = NixConfig::from_nix(&crate::command::NixCmd::default()).await?;
    println!("Max Jobs: {}", v.max_jobs.value);
    Ok(())
}
