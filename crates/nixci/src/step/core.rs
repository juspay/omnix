//! All CI steps available in nixci
use clap::Parser;
use nix_rs::{command::NixCmd, flake::url::FlakeUrl};
use serde::Deserialize;

use super::{
    build::{BuildStep, BuildStepArgs},
    flake_check::FlakeCheckStep,
    lockfile::LockfileStep,
};
use crate::command::run::RunCommand;
use crate::config::subflake::SubflakeConfig;

/// CI steps to run
///
/// Contains some builtin steps, as well as custom steps (defined by user)
#[derive(Debug, Default, Deserialize)]
pub struct Steps {
    /// [LockfileStep]
    #[serde(default, rename = "lockfile")]
    pub lockfile_step: LockfileStep,

    /// [BuildStep]
    #[serde(default, rename = "build")]
    pub build_step: BuildStep,

    /// [FlakeCheckStep]
    #[serde(default, rename = "flake-check")]
    pub flake_check_step: FlakeCheckStep,
    // TODO: custom steps
}

/// CLI arguments associated with [Steps]
#[derive(Parser, Debug, Clone)]
pub struct StepsArgs {
    /// [BuildStepArgs]
    #[command(flatten)]
    pub build_step_args: BuildStepArgs,
}

impl Steps {
    /// Run all CI steps
    pub async fn run(
        &self,
        cmd: &NixCmd,
        verbose: bool,
        run_cmd: &RunCommand,
        url: &FlakeUrl,
        subflake: &SubflakeConfig,
    ) -> anyhow::Result<()> {
        self.lockfile_step.run(cmd, url, subflake).await?;

        let res = self
            .build_step
            .run(cmd, verbose, run_cmd, url, subflake)
            .await?;
        // TODO: Support --json for structured output grouped by steps
        res.print();

        self.flake_check_step.run(cmd, url, subflake).await?;

        Ok(())
    }
}

impl StepsArgs {
    /// Convert this type back to the user-facing command line arguments
    pub fn to_cli_args(&self) -> Vec<String> {
        self.build_step_args.to_cli_args()
    }
}
