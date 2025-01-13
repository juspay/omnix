//! The cachix step
use colored::Colorize;
use nix_rs::{
    command::NixCmd,
    flake::{self, command::FlakeOptions, url::FlakeUrl},
};
use serde::Deserialize;

use crate::config::subflake::SubflakeConfig;

/// Run `nix flake check`
///
/// Note: `nix build ...` does not evaluate all the checks that `nix flake check` does. So, enabling this steps allows `om ci` to run those evaluation checks.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct FlakeCheckStep {
    /// Whether to enable this step
    ///
    /// Disabled by default, since only a handful of flakes need this (for others, it will unnecessarily slow down the build)
    pub enable: bool,
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
        let opts = FlakeOptions {
            override_inputs: subflake.override_inputs.clone(),
            ..Default::default()
        };
        flake::command::check(nixcmd, &opts, &sub_flake_url).await?;
        Ok(())
    }
}
