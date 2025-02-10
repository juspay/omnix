//! Rust module for `nix show-config`

use std::{convert::Infallible, str::FromStr};

use serde::{Deserialize, Serialize};
use serde_with::DeserializeFromStr;
use tokio::sync::OnceCell;
use tracing::instrument;
use url::Url;

use crate::{
    command::{NixCmd, NixCmdError},
    version::NixVersion,
};

use super::flake::system::System;

/// Nix configuration spit out by `nix show-config`
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct NixConfig {
    /// Number of CPU cores used for nix builds
    pub cores: ConfigVal<i32>,
    /// Experimental features currently enabled
    pub experimental_features: ConfigVal<Vec<String>>,
    /// Extra platforms to build for
    pub extra_platforms: ConfigVal<Vec<String>>,
    /// The flake registry to use to lookup atomic flake inputs
    pub flake_registry: ConfigVal<String>,
    /// Maximum number of jobs to run in parallel
    pub max_jobs: ConfigVal<i32>,
    /// Cache substituters
    pub substituters: ConfigVal<Vec<Url>>,
    /// Current system
    pub system: ConfigVal<System>,
    /// Trusted users
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

static NIX_CONFIG: OnceCell<Result<NixConfig, NixConfigError>> = OnceCell::const_new();

static NIX_2_20_0: NixVersion = NixVersion {
    major: 2,
    minor: 20,
    patch: 0,
};

impl NixConfig {
    /// Get the once version of `NixConfig`.
    #[instrument(name = "show-config(once)")]
    pub async fn get() -> &'static Result<NixConfig, NixConfigError> {
        NIX_CONFIG
            .get_or_init(|| async {
                let mut cmd = NixCmd::default();
                cmd.args.with_nix_command(); // Enable nix-command, since don't yet know if it is already enabled.
                let nix_ver = NixVersion::get().await.as_ref()?;
                let cfg = NixConfig::from_nix(&cmd, nix_ver).await?;
                Ok(cfg)
            })
            .await
    }

    /// Get the output of `nix show-config`
    #[instrument(name = "show-config")]
    pub async fn from_nix(
        nix_cmd: &super::command::NixCmd,
        nix_version: &NixVersion,
    ) -> Result<NixConfig, super::command::NixCmdError> {
        let args: Vec<&str> = if nix_version >= &NIX_2_20_0 {
            vec!["config", "show", "--json"]
        } else {
            vec!["show-config", "--json"]
        };
        let v = nix_cmd.run_with_args_expecting_json(&args).await?;
        Ok(v)
    }

    /// Is flakes and command features enabled?
    pub fn is_flakes_enabled(&self) -> bool {
        self.experimental_features
            .value
            .contains(&"nix-command".to_string())
            && self
                .experimental_features
                .value
                .contains(&"flakes".to_string())
    }
}

/// Error type for `NixConfig`
#[derive(thiserror::Error, Debug)]
pub enum NixConfigError {
    /// A [NixCmdError]
    #[error("Nix command error: {0}")]
    NixCmdError(#[from] NixCmdError),

    /// A [NixCmdError] with a static lifetime
    #[error("Nix command error: {0}")]
    NixCmdErrorStatic(#[from] &'static NixCmdError),
}

/// Accepted value for "trusted-users" in nix.conf
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

    /// Display the nix.conf original string
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
    let v = NixConfig::get().await.as_ref().unwrap();
    println!("Max Jobs: {}", v.max_jobs.value);
    Ok(())
}
