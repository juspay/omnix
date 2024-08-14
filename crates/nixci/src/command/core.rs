use clap::Subcommand;
use colored::Colorize;
use nix_rs::{command::NixCmd, store::StorePath};
use tracing::instrument;

use crate::{config, flake_ref::FlakeRef, github, nix::devour_flake};

use super::{build::BuildCommand, gh::GHMatrixCommand};

#[derive(Debug, Subcommand, Clone)]
pub enum Command {
    /// Build all outputs of a flake
    Build(BuildCommand),

    /// Print the Github Actions matrix configuration as JSON
    #[clap(name = "gh-matrix")]
    DumpGithubActionsMatrix(GHMatrixCommand),
}

impl Default for Command {
    fn default() -> Self {
        Self::Build(Default::default())
    }
}

impl Command {
    // Pre-process `Command`
    pub fn preprocess(&mut self) {
        // Adjust to devour_flake's expectations
        if let Command::Build(build_cfg) = self {
            devour_flake::transform_override_inputs(&mut build_cfg.extra_nix_build_args);
        }
    }

    #[instrument(name = "run", skip(self))]
    pub async fn run(self, nixcmd: &NixCmd, verbose: bool) -> anyhow::Result<Vec<StorePath>> {
        tracing::info!("{}", format!("\n👟 Reading om.ci config from flake").bold());
        let cfg = self.get_config(nixcmd).await?;
        match self {
            Command::Build(cmd) => cmd.run(nixcmd, verbose, cfg).await,
            Command::DumpGithubActionsMatrix(cmd) => {
                let matrix =
                    github::matrix::GitHubMatrix::from(cmd.systems.clone(), &cfg.subflakes);
                println!("{}", serde_json::to_string(&matrix)?);
                Ok(vec![])
            }
        }
    }

    /// Get the nixci [config::Config] associated with this subcommand
    async fn get_config(&self, cmd: &NixCmd) -> anyhow::Result<config::Config> {
        let url = self.get_flake_ref().to_flake_url().await?;
        let cfg = config::Config::from_flake_url(cmd, &url).await?;
        tracing::debug!("Config: {cfg:?}");
        Ok(cfg)
    }

    /// Get the flake ref associated with this subcommand
    fn get_flake_ref(&self) -> FlakeRef {
        match self {
            Command::Build(cmd) => cmd.flake_ref.clone(),
            Command::DumpGithubActionsMatrix(cmd) => cmd.flake_ref.clone(),
        }
    }
}
