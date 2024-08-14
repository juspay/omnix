use anyhow::{Context, Result};
use clap::Parser;
use colored::Colorize;
use nix_rs::{command::NixCmd, config::NixConfig, flake::system::System, info::NixInfo};

use crate::{
    config,
    flake_ref::FlakeRef,
    nix::system_list::{SystemsList, SystemsListFlakeRef},
    step,
};

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

impl Default for BuildCommand {
    fn default() -> Self {
        BuildCommand::parse_from::<[_; 0], &str>([])
    }
}

impl BuildCommand {
    pub async fn run(
        &self,
        nixcmd: &NixCmd,
        verbose: bool,
        cfg: config::core::Config,
    ) -> anyhow::Result<()> {
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
        for out in &outs {
            println!("{}", out);
        }
        Ok(())
    }

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
