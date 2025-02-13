//! The `om ci` subcommands
use clap::Subcommand;
use colored::Colorize;
use nix_rs::command::NixCmd;
use omnix_common::config::OmConfig;
use tracing::instrument;

use crate::flake_ref::FlakeRef;

use super::{gh_matrix::GHMatrixCommand, run::RunCommand};

/// Top-level commands for `om ci`
#[derive(Debug, Subcommand, Clone)]
pub enum Command {
    /// Run all CI steps for all or given subflakes
    Run(RunCommand),

    /// Print the Github Actions matrix configuration as JSON
    #[clap(name = "gh-matrix")]
    DumpGithubActionsMatrix(GHMatrixCommand),
}

impl Default for Command {
    fn default() -> Self {
        Self::Run(Default::default())
    }
}

impl Command {
    /// Run the command
    #[instrument(name = "run", skip(self))]
    pub async fn run(self, verbose: bool) -> anyhow::Result<()> {
        tracing::info!("{}", "\n👟 Reading om.ci config from flake".bold());
        let url = self.get_flake_ref().to_flake_url().await?;
        let cfg = OmConfig::get(self.nixcmd(), &url).await?;

        tracing::debug!("OmConfig: {cfg:?}");
        match self {
            Command::Run(cmd) => cmd.run(verbose, cfg).await,
            Command::DumpGithubActionsMatrix(cmd) => cmd.run(cfg).await,
        }
    }

    fn nixcmd(&self) -> &NixCmd {
        match self {
            Command::Run(cmd) => &cmd.nixcmd,
            Command::DumpGithubActionsMatrix(cmd) => &cmd.nixcmd,
        }
    }

    /// Get the [FlakeRef] associated with this subcommand
    fn get_flake_ref(&self) -> &FlakeRef {
        match self {
            Command::Run(cmd) => &cmd.flake_ref,
            Command::DumpGithubActionsMatrix(cmd) => &cmd.flake_ref,
        }
    }

    /// Convert this type back to the user-facing command line arguments
    pub fn to_cli_args(&self) -> Vec<String> {
        let mut args = vec!["ci".to_string(), "run".to_string()];
        match self {
            Command::Run(cmd) => {
                args.extend(cmd.to_cli_args());
            }
            Command::DumpGithubActionsMatrix(_cmd) => {
                unimplemented!("Command::DumpGithubActionsMatrix::to_cli_args")
            }
        }
        args
    }
}
