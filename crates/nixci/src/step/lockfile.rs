//! The lockfile step
use colored::Colorize;
use nix_rs::{command::NixCmd, flake::url::FlakeUrl};
use serde::Deserialize;

use crate::{config::subflake::SubflakeConfig, nix};

/// Check that `flake.lock` is not out of date.
#[derive(Debug, Clone, Deserialize)]
pub struct LockfileStep {
    /// Whether to enable this step
    pub enable: bool,
}

impl Default for LockfileStep {
    fn default() -> Self {
        LockfileStep { enable: true }
    }
}

impl LockfileStep {
    /// Run this step
    pub async fn run(
        &self,
        nixcmd: &NixCmd,
        url: &FlakeUrl,
        subflake: &SubflakeConfig,
    ) -> anyhow::Result<()> {
        if subflake.override_inputs.is_empty() {
            tracing::info!(
                "{}",
                format!("🫀 Checking that {}/flake.lock is up-to-date", subflake.dir).bold()
            );
            let sub_flake_url = url.sub_flake_url(subflake.dir.clone());
            nix::lock::nix_flake_lock_check(nixcmd, &sub_flake_url).await?;
        }
        Ok(())
    }
}
