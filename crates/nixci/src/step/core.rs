use clap::Parser;
use nix_rs::{command::NixCmd, flake::url::FlakeUrl};
use serde::Deserialize;

use crate::command::run::RunCommand;
use crate::config::subflake::SubflakeConfig;
use crate::step::{
    build::{BuildStep, BuildStepArgs},
    lockfile::LockfileStep,
};

/// CI steps to run
///
/// Contains some builtin steps, as well as custom steps (defined by user)
#[derive(Debug, Default, Deserialize)]
pub struct Steps {
    pub lockfile_step: LockfileStep,
    pub build_step: BuildStep,
    // TODO: custom steps
}

#[derive(Parser, Debug, Clone)]
pub struct StepsArgs {
    #[command(flatten)]
    pub build_step_args: BuildStepArgs,
}

impl Steps {
    pub async fn run(
        &self,
        cmd: &NixCmd,
        verbose: bool,
        run_cmd: &RunCommand,
        url: &FlakeUrl,
        subflake: &SubflakeConfig,
    ) -> anyhow::Result<()> {
        self.lockfile_step.run(cmd, url, subflake).await?;
        self.build_step
            .run(cmd, verbose, run_cmd, url, subflake)
            .await?;
        Ok(())
    }
}
