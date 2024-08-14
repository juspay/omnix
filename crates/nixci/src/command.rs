use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use colored::Colorize;
use nix_rs::{
    command::NixCmd, config::NixConfig, flake::system::System, info::NixInfo, store::StorePath,
};
use tracing::instrument;

use crate::{
    config,
    flake_ref::FlakeRef,
    github,
    nix::{
        devour_flake,
        system_list::{SystemsList, SystemsListFlakeRef},
    },
    step,
};

#[derive(Debug, Subcommand, Clone)]
pub enum Command {
    /// Build all outputs of a flake
    Build(BuildCommand),

    /// Print the Github Actions matrix configuration as JSON
    #[clap(name = "gh-matrix")]
    DumpGithubActionsMatrix(GHMatrixCommand),
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
        let cfg = self.get_config(nixcmd).await?;
        match self {
            Command::Build(cmd) => {
                tracing::info!("{}", format!("🍏 Building {}", cmd.flake_ref).bold());
                let nix_config = NixConfig::get().await.as_ref()?;
                let nix_info = NixInfo::new(nix_config.clone())
                    .await
                    .with_context(|| "Unable to gather nix info")?;
                // First, run the necessary health checks
                step::nix_version::check_nix_version(&cfg.ref_.flake_url, &nix_info).await?;
                // Then, do the build
                step::build::nixci_build(nixcmd, verbose, &cmd, &cfg, &nix_info.nix_config).await
            }
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

#[derive(Parser, Debug, Clone)]
pub struct BuildCommand {
    /// The systems list to build for. If empty, build for current system.
    ///
    /// Must be a flake reference which, when imported, must return a Nix list
    /// of systems. You may use one of the lists from
    /// <https://github.com/nix-systems>.
    #[arg(long, default_value = "github:nix-systems/empty")]
    pub systems: SystemsListFlakeRef,

    /// Flake URL or github URL
    ///
    /// A specific nixci` configuration can be specified
    /// using '#': e.g. `nixci .#extra-tests`
    #[arg(default_value = ".")]
    pub flake_ref: FlakeRef,

    /// Additional arguments to pass through to `nix build`
    #[arg(last = true, default_values_t = vec![
    "--refresh".to_string(),
    "-j".to_string(),
    "auto".to_string(),
    ])]
    pub extra_nix_build_args: Vec<String>,

    /// Print build and runtime dependencies along with out paths
    ///
    /// By default, `nixci build` prints only the out paths. This option is
    /// useful to explicitly push all dependencies to a cache.
    #[clap(long, short = 'd')]
    pub print_all_dependencies: bool,
}

impl BuildCommand {
    pub async fn get_systems(&self, cmd: &NixCmd, nix_config: &NixConfig) -> Result<Vec<System>> {
        let systems = SystemsList::from_flake(cmd, &self.systems).await?.0;
        if systems.is_empty() {
            let current_system = &nix_config.system.value;
            Ok(vec![current_system.clone()])
        } else {
            Ok(systems)
        }
    }
}

#[derive(Parser, Debug, Clone)]
pub struct GHMatrixCommand {
    /// Flake URL or github URL
    ///
    /// A specific nixci configuration can be specified
    /// using '#': e.g. `nixci .#extra-tests`
    #[arg(default_value = ".")]
    pub flake_ref: FlakeRef,

    /// Systems to include in the matrix
    #[arg(long, value_parser, value_delimiter = ',')]
    pub systems: Vec<System>,
}
