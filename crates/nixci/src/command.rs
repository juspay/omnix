use anyhow::Result;
use clap::{Parser, Subcommand};
use colored::Colorize;
use nix_rs::{command::NixCmd, config::NixConfig, flake::system::System};

use crate::{
    config,
    flake_ref::FlakeRef,
    nix::{
        devour_flake,
        system_list::{SystemsList, SystemsListFlakeRef},
    },
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

    /// Get the nixci [config::Config] associated with this subcommand
    pub async fn get_config(cmd: &NixCmd, flake_ref: &FlakeRef) -> anyhow::Result<config::Config> {
        let url = flake_ref.to_flake_url().await?;
        tracing::info!("{}", format!("🍏 Building {}", url.0).bold());
        let cfg = config::Config::from_flake_url(cmd, &url).await?;
        tracing::debug!("Config: {cfg:?}");
        Ok(cfg)
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
