//! Rust module for `nix show-config`

use serde::{Deserialize, Serialize};

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
    pub trusted_users: ConfigVal<Vec<String>>,
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

#[tokio::test]
async fn test_nix_config() -> Result<(), crate::command::NixCmdError> {
    let v = NixConfig::from_nix(&crate::command::NixCmd::default()).await?;
    println!("Max Jobs: {}", v.max_jobs.value);
    Ok(())
}
