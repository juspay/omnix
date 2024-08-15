use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::Parser;
use colored::Colorize;
use nix_rs::{command::NixCmd, config::NixConfig, flake::system::System, info::NixInfo};

use crate::{
    config::core::Config,
    flake_ref::FlakeRef,
    nix::{
        devour_flake, ssh,
        system_list::{SystemsList, SystemsListFlakeRef},
    },
    step,
};

/// We expect this environment to be set in Nix build and shell.
pub const OMNIX_SOURCE: &str = env!("OMNIX_SOURCE");

/// Build all outputs of a flake
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

    /// Run om ci build remotely
    #[clap(long)]
    pub on: Option<String>,
}

impl Default for BuildCommand {
    fn default() -> Self {
        BuildCommand::parse_from::<[_; 0], &str>([])
    }
}

impl BuildCommand {
    pub fn preprocess(&mut self) {
        // Adjust to devour_flake's expectations
        devour_flake::transform_override_inputs(&mut self.extra_nix_build_args);
    }

    /// Run the build command
    pub async fn run(&self, nixcmd: &NixCmd, verbose: bool, cfg: Config) -> anyhow::Result<()> {
        // TODO: We'll refactor this function to use steps
        // https://github.com/juspay/omnix/issues/216

        tracing::info!("{}", format!("\n👟 Gathering NixInfo").bold());
        let nix_info = NixInfo::get()
            .await
            .as_ref()
            .with_context(|| "Unable to gather nix info")?;

        // First, run the necessary health checks
        tracing::info!("{}", format!("\n🫀 Performing health check").bold());
        step::nix_version::check_nix_version(&cfg.ref_.flake_url, &nix_info).await?;

        // Then, do the build
        tracing::info!("{}", format!("\n🍏 Building {}", self.flake_ref).bold());
        let outs =
            step::build::build_flake(nixcmd, verbose, &self, &cfg, &nix_info.nix_config).await?;

        // Print the outputs
        for out in &outs {
            println!("{}", out);
        }
        Ok(())
    }

    /// Run the build command on remote
    pub async fn run_remote(
        &self,
        nixcmd: &NixCmd,
        cfg: Config,
        host: &String,
    ) -> anyhow::Result<()> {
        let metadata = nix_rs::flake::metadata::from_nix(nixcmd, &cfg.ref_.flake_url).await?;

        let omnix_source = PathBuf::from(OMNIX_SOURCE);

        let args = vec![&omnix_source, &metadata.path];

        nix_rs::copy::from_nix(nixcmd, &host, args).await?;

        let _ = ssh::on_ssh(&self, &host, &omnix_source, metadata.path, cfg.ref_).await?;

        Ok(())
    }

    /// Get the systems to build for
    pub async fn get_systems(&self, cmd: &NixCmd, nix_config: &NixConfig) -> Result<Vec<System>> {
        let systems = SystemsList::from_flake(cmd, &self.systems).await?.0;
        if systems.is_empty() {
            // An empty systems list means build for the current system
            let current_system = &nix_config.system.value;
            Ok(vec![current_system.clone()])
        } else {
            Ok(systems)
        }
    }
}
