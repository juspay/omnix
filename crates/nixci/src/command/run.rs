//! The run command
use anyhow::{Context, Result};
use clap::Parser;
use colored::Colorize;
use nix_health::{traits::Checkable, NixHealth};
use nix_rs::{
    command::NixCmd,
    config::NixConfig,
    flake::{system::System, url::FlakeUrl},
    info::NixInfo,
};

use crate::{
    config::core::Config,
    flake_ref::FlakeRef,
    nix::system_list::{SystemsList, SystemsListFlakeRef},
};

/// Command to run all CI steps
#[derive(Parser, Debug, Clone)]
pub struct RunCommand {
    /// The systems list to build for. If empty, build for current system.
    ///
    /// Must be a flake reference which, when imported, must return a Nix list
    /// of systems. You may use one of the lists from
    /// <https://github.com/nix-systems>.
    #[arg(long, default_value = "github:nix-systems/empty")]
    pub systems: SystemsListFlakeRef,

    /// Flake URL or github URL
    ///
    /// A specific configuration can be specified
    /// using '#': e.g. `om ci run .#default.extra-tests`
    #[arg(default_value = ".")]
    pub flake_ref: FlakeRef,

    /// Arguments for all steps
    #[command(flatten)]
    pub steps_args: crate::step::core::StepsArgs,
}

impl Default for RunCommand {
    fn default() -> Self {
        RunCommand::parse_from::<[_; 0], &str>([])
    }
}

impl RunCommand {
    /// Preprocess this command
    pub fn preprocess(&mut self) {
        self.steps_args.build_step_args.preprocess();
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
        check_nix_version(&cfg.ref_.flake_url, &nix_info).await?;

        // Then, do the CI steps
        tracing::info!(
            "{}",
            format!("\n🤖 Running CI for {}", self.flake_ref).bold()
        );
        ci_run(nixcmd, verbose, &self, &cfg, &nix_info.nix_config).await?;

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

/// Check that Nix version is not too old.
pub async fn check_nix_version(flake_url: &FlakeUrl, nix_info: &NixInfo) -> anyhow::Result<()> {
    let nix_health = NixHealth::from_flake(flake_url).await?;
    let checks = nix_health.nix_version.check(nix_info, Some(flake_url));
    let exit_code = NixHealth::print_report_returning_exit_code(&checks);

    if exit_code != 0 {
        std::process::exit(exit_code);
    }
    Ok(())
}

/// Run CI fo all subflakes
pub async fn ci_run(
    cmd: &NixCmd,
    verbose: bool,
    run_cmd: &RunCommand,
    cfg: &Config,
    nix_config: &NixConfig,
) -> anyhow::Result<()> {
    let systems = run_cmd.get_systems(cmd, nix_config).await?;

    for (subflake_name, subflake) in &cfg.subflakes.0 {
        let name = format!("{}.{}", cfg.ref_.selected_name, subflake_name).italic();

        if subflake.skip {
            tracing::info!("🍊 {} {}", name, "skipped (deselected out)".dimmed());
            continue;
        }

        let compatible_system = subflake.can_build_on(&systems);
        if !compatible_system {
            tracing::info!(
                "🍊 {} {}",
                name,
                "skipped (cannot build on this system)".dimmed()
            );
            continue;
        }

        tracing::info!("🍎 {}", name);
        subflake
            .steps
            .run(cmd, verbose, run_cmd, &cfg.ref_.flake_url, subflake)
            .await?;
    }

    Ok(())
}
