//! The run command
use std::{collections::HashMap, path::PathBuf};

use anyhow::{Context, Result};
use clap::Parser;
use colored::Colorize;
use nix_health::{traits::Checkable, NixHealth};
use nix_rs::{
    command::NixCmd,
    config::NixConfig,
    flake::{system::System, url::FlakeUrl},
    info::NixInfo,
    store::uri::StoreURI,
};
use omnix_common::config::OmConfig;

use crate::{
    config::subflakes::SubflakesConfig,
    flake_ref::FlakeRef,
    nix::system_list::{SystemsList, SystemsListFlakeRef},
    step::core::StepsResult,
};

use super::run_remote;

/// Run all CI steps for all or given subflakes
/// Command to run all CI steps
#[derive(Parser, Debug, Clone)]
pub struct RunCommand {
    /// Run `om ci run` remotely on the given store URI
    #[clap(long)]
    pub on: Option<StoreURI>,

    /// The systems list to build for. If empty, build for current system.
    ///
    /// Must be a flake reference which, when imported, must return a Nix list
    /// of systems. You may use one of the lists from
    /// <https://github.com/nix-systems>.
    #[arg(long)]
    pub systems: Option<SystemsListFlakeRef>,

    /// Path to write the results of the CI run (in JSON) to
    #[arg(long, short = 'o')]
    pub write_results: Option<PathBuf>,

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
    pub async fn run(
        &self,
        nixcmd: &NixCmd,
        verbose: bool,
        cfg: OmConfig<SubflakesConfig>,
    ) -> anyhow::Result<()> {
        match &self.on {
            Some(store_uri) => run_remote::run_on_remote_store(nixcmd, self, &cfg, store_uri).await,
            None => self.run_local(nixcmd, verbose, cfg).await,
        }
    }

    /// Run [RunCommand] on local Nix store.
    async fn run_local(
        &self,
        nixcmd: &NixCmd,
        verbose: bool,
        cfg: OmConfig<SubflakesConfig>,
    ) -> anyhow::Result<()> {
        // TODO: We'll refactor this function to use steps
        // https://github.com/juspay/omnix/issues/216

        tracing::info!("{}", "\n👟 Gathering NixInfo".bold());
        let nix_info = NixInfo::get()
            .await
            .as_ref()
            .with_context(|| "Unable to gather nix info")?;

        // First, run the necessary health checks
        tracing::info!("{}", "\n🫀 Performing health check".bold());
        check_nix_version(&cfg.flake_url, nix_info).await?;

        // Then, do the CI steps
        tracing::info!(
            "{}",
            format!("\n🤖 Running CI for {}", self.flake_ref).bold()
        );
        let res = ci_run(nixcmd, verbose, self, &cfg, &nix_info.nix_config).await?;

        if let Some(results_file) = self.write_results.as_ref() {
            serde_json::to_writer(std::fs::File::create(results_file)?, &res)?;
            tracing::info!(
                "Results written to {}",
                results_file.to_string_lossy().bold()
            );
        } else {
            for (_, result) in res {
                result.print();
            }
        }

        Ok(())
    }

    /// Get the systems to build for
    pub async fn get_systems(&self, cmd: &NixCmd, nix_config: &NixConfig) -> Result<Vec<System>> {
        match &self.systems {
            None => {
                // An empty systems list means build for the current system
                let current_system = &nix_config.system.value;
                Ok(vec![current_system.clone()])
            }
            Some(systems) => {
                let systems = SystemsList::from_flake(cmd, systems).await?.0;
                Ok(systems)
            }
        }
    }

    /// Convert this type back to the user-facing command line arguments
    pub fn to_cli_args(&self) -> Vec<String> {
        let mut args = vec![];

        if let Some(uri) = self.on.as_ref() {
            args.push("--on".to_owned());
            args.push(uri.to_string());
        }

        if let Some(systems) = self.systems.as_ref() {
            args.push("--systems".to_string());
            args.push(systems.0 .0.clone());
        }

        args.push(self.flake_ref.to_string());

        args.extend(self.steps_args.to_cli_args());

        args
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
    cfg: &OmConfig<SubflakesConfig>,
    nix_config: &NixConfig,
) -> anyhow::Result<HashMap<String, StepsResult>> {
    let mut res = HashMap::new();
    let systems = run_cmd.get_systems(cmd, nix_config).await?;

    let (config, attrs) = cfg.get_referenced()?;
    // User's filter by subflake name
    let only_subflake = attrs.first();

    for (subflake_name, subflake) in &config.0 {
        let name = subflake_name.italic();

        if let Some(s) = only_subflake
            && s != subflake_name
        {
            tracing::info!("\n🍊 {} {}", name, "skipped (deselected out)".dimmed());
            continue;
        }

        let compatible_system = subflake.can_run_on(&systems);
        if !compatible_system {
            tracing::info!(
                "\n🍊 {} {}",
                name,
                "skipped (cannot run on this system)".dimmed()
            );
            continue;
        }

        tracing::info!("\n🍎 {}", name);
        let steps_res = subflake
            .steps
            .run(cmd, verbose, run_cmd, &systems, &cfg.flake_url, subflake)
            .await?;
        res.insert(subflake_name.clone(), steps_res);
    }

    tracing::info!("\n🥳 Success!");

    Ok(res)
}
