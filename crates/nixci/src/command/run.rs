//! The run command
use anyhow::{Context, Result};
use clap::Parser;
use colored::Colorize;
use nix_health::{traits::Checkable, NixHealth};
use nix_rs::{
    command::NixCmd,
    config::NixConfig,
    flake::{metadata::FlakeMetadata, system::System, url::FlakeUrl},
    info::NixInfo,
};
use std::path::PathBuf;

use crate::{
    config::{core::Config, ref_::ConfigRef},
    flake_ref::FlakeRef,
    nix::{
        ssh,
        system_list::{SystemsList, SystemsListFlakeRef},
    },
};

/// Path to Rust source corresponding to this (running) instance of Omnix
pub const OMNIX_SOURCE: &str = env!("OMNIX_SOURCE");

/// Run all CI steps for all or given subflakes
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

    /// Run the build command which decides whether to do ci run on current machine or a remote machine
    pub async fn run(&self, nixcmd: &NixCmd, verbose: bool, cfg: Config) -> anyhow::Result<()> {
        match &self.steps_args.build_step_args.on {
            Some(host) => self.run_remote(nixcmd, cfg, host).await,
            None => self.run_local(nixcmd, verbose, cfg).await,
        }
    }

    /// Runs the ci run steps on current machine
    async fn run_local(&self, nixcmd: &NixCmd, verbose: bool, cfg: Config) -> anyhow::Result<()> {
        // TODO: We'll refactor this function to use steps
        // https://github.com/juspay/omnix/issues/216

        tracing::info!("{}", "\n👟 Gathering NixInfo".bold());
        let nix_info = NixInfo::get()
            .await
            .as_ref()
            .with_context(|| "Unable to gather nix info")?;

        // First, run the necessary health checks
        tracing::info!("{}", "\n🫀 Performing health check".bold());
        check_nix_version(&cfg.ref_.flake_url, nix_info).await?;

        // Then, do the CI steps
        tracing::info!(
            "{}",
            format!("\n🤖 Running CI for {}", self.flake_ref).bold()
        );
        ci_run(nixcmd, verbose, self, &cfg, &nix_info.nix_config).await?;

        Ok(())
    }

    /// Run the ci run steps on remote
    async fn run_remote(&self, nixcmd: &NixCmd, cfg: Config, host: &str) -> anyhow::Result<()> {
        let metadata = FlakeMetadata::from_nix(nixcmd, &cfg.ref_.flake_url).await?;

        let omnix_source = PathBuf::from(OMNIX_SOURCE);

        nix_rs::copy::nix_copy(nixcmd, host, &[omnix_source.clone(), metadata.path.clone()])
            .await?;

        let ci_run_args = self.get_ci_run_args_for_remote(metadata.path, cfg.ref_)?;

        let nix_run_args: Vec<String> = vec![
            "nix run".to_string(),
            format!("{}#default", OMNIX_SOURCE),
            "--".to_string(),
        ]
        .into_iter()
        .chain(ci_run_args.into_iter())
        .collect();

        // call ci run on remote machine through ssh
        ssh::on_ssh(host, &nix_run_args).await?;

        Ok(())
    }

    // Return ci run args along with build_step_args
    fn get_ci_run_args_for_remote(
        &self,
        flake_url: PathBuf,
        cfg_ref: ConfigRef,
    ) -> Result<Vec<String>> {
        let mut flake_to_build = flake_url.to_string_lossy().as_ref().to_string();

        // add sub-flake if selected to be built
        if let Some(sub_flake) = cfg_ref.selected_subflake {
            flake_to_build
                .push_str(&format!("#{}.{}", cfg_ref.selected_name, sub_flake).to_string());
        }

        let mut nix_run_args = vec![
            "ci".to_string(),
            "run".to_string(),
            flake_to_build.to_string(),
        ];

        // Add print-all-dependencies flag if passed
        if self.steps_args.build_step_args.print_all_dependencies {
            nix_run_args.push("--print-all-dependencies".to_string());
        }

        // Add extra nix build arguments
        nix_run_args.push("--".to_string());
        nix_run_args.extend(
            self.steps_args
                .build_step_args
                .extra_nix_build_args
                .iter()
                .cloned(),
        );

        Ok(nix_run_args)
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
