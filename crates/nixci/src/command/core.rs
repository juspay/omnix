//! The nixci commands
use clap::Subcommand;
use colored::Colorize;
use nix_rs::command::NixCmd;
use tracing::instrument;

use crate::{config::core::Config, flake_ref::FlakeRef};

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
    /// Pre-process `Command`
    pub fn preprocess(&mut self) {
        if let Command::Run(cmd) = self {
            cmd.preprocess()
        }
    }

    /// Run the command
    #[instrument(name = "run", skip(self))]
    pub async fn run(self, nixcmd: &NixCmd, verbose: bool) -> anyhow::Result<()> {
        tracing::info!("{}", "\n👟 Reading om.ci config from flake".bold());
        let cfg = self.get_config(nixcmd).await?;
        match self {
            Command::Run(cmd) => cmd.run(nixcmd, verbose, cfg).await,
            Command::DumpGithubActionsMatrix(cmd) => cmd.run(cfg).await,
        }
    }

    /// Get the nixci [config::Config] associated with this subcommand
    async fn get_config(&self, cmd: &NixCmd) -> anyhow::Result<Config> {
        let url = self.get_flake_ref().to_flake_url().await?;
        let cfg = Config::from_flake_url(cmd, &url).await?;
        tracing::debug!("Config: {cfg:?}");
        Ok(cfg)
    }

    /// Get the flake ref associated with this subcommand
    fn get_flake_ref(&self) -> FlakeRef {
        match self {
            Command::Run(cmd) => cmd.flake_ref.clone(),
            Command::DumpGithubActionsMatrix(cmd) => cmd.flake_ref.clone(),
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
