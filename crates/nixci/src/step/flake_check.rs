//! The cachix step
use colored::Colorize;
use nix_rs::{command::NixCmd, flake::url::FlakeUrl};
use serde::Deserialize;

use crate::config::subflake::SubflakeConfig;

/// Run `nix flake check`
#[derive(Debug, Deserialize)]
pub struct FlakeCheckStep {
    /// Whether to enable this step
    pub enable: bool,
}

impl Default for FlakeCheckStep {
    fn default() -> Self {
        FlakeCheckStep { enable: true }
    }
}

impl FlakeCheckStep {
    /// Run this step
    pub async fn run(
        &self,
        nixcmd: &NixCmd,
        url: &FlakeUrl,
        subflake: &SubflakeConfig,
    ) -> anyhow::Result<()> {
        tracing::info!(
            "{}",
            format!("🩺 Running flake check on: {}", subflake.dir).bold()
        );
        let sub_flake_url = url.sub_flake_url(subflake.dir.clone());
        nixcmd
            .run_with_args(["flake", "check", &sub_flake_url])
            .await?;
        Ok(())
    }
}
