//! The run command
use std::{
    collections::HashMap,
    env,
    io::Write,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use clap::Parser;
use colored::Colorize;
use nix_rs::{
    command::NixCmd,
    config::NixConfig,
    flake::{functions::addstringcontext, system::System, url::FlakeUrl},
    info::NixInfo,
    store::{path::StorePath, uri::StoreURI},
    system_list::{SystemsList, SystemsListFlakeRef},
};
use omnix_common::config::OmConfig;
use omnix_health::{traits::Checkable, NixHealth};
use serde::{Deserialize, Serialize};

use crate::{
    config::subflakes::SubflakesConfig, flake_ref::FlakeRef, github::actions::in_github_log_group,
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
    ///
    /// You can also pass the individual system name, if they are supported by omnix.
    #[arg(long)]
    pub systems: Option<SystemsListFlakeRef>,

    /// Symlink to build results (as JSON)
    #[arg(
        long,
        short = 'o',
        default_value = "result",
        conflicts_with = "no_link",
        alias = "results", // For backwards compat
        name = "PATH"
    )]
    out_link: Option<PathBuf>,

    /// Do not create a symlink to build results JSON
    #[arg(long)]
    no_link: bool,

    /// Flake URL or github URL
    ///
    /// A specific configuration can be specified
    /// using '#': e.g. `om ci run .#default.extra-tests`
    #[arg(default_value = ".")]
    pub flake_ref: FlakeRef,

    /// Print Github Actions log groups (enabled by default when run in Github Actions)
    #[clap(long, default_value_t = env::var("GITHUB_ACTION").is_ok())]
    pub github_output: bool,

    /// Arguments for all steps
    #[command(flatten)]
    pub steps_args: crate::step::core::StepsArgs,

    /// Nix command global options
    #[command(flatten)]
    pub nixcmd: NixCmd,
}

impl Default for RunCommand {
    fn default() -> Self {
        RunCommand::parse_from::<[_; 0], &str>([])
    }
}

impl RunCommand {
    /// Get the out-link path
    pub fn get_out_link(&self) -> Option<&Path> {
        if self.no_link {
            None
        } else {
            self.out_link.as_ref().map(PathBuf::as_ref)
        }
    }

    /// Override the `flake_ref` and `out_link`` for building locally.
    pub fn local_with(&self, flake_ref: FlakeRef, out_link: Option<PathBuf>) -> Self {
        let mut new = self.clone();
        new.on = None; // Disable remote building
        new.flake_ref = flake_ref;
        new.no_link = out_link.is_none();
        new.out_link = out_link;
        new
    }

    /// Run the build command which decides whether to do ci run on current machine or a remote machine
    pub async fn run(&self, verbose: bool, cfg: OmConfig) -> anyhow::Result<()> {
        match &self.on {
            Some(store_uri) => {
                run_remote::run_on_remote_store(&self.nixcmd, self, &cfg, store_uri).await
            }
            None => self.run_local(verbose, cfg).await,
        }
    }

    /// Run [RunCommand] on local Nix store.
    async fn run_local(&self, verbose: bool, cfg: OmConfig) -> anyhow::Result<()> {
        // TODO: We'll refactor this function to use steps
        // https://github.com/juspay/omnix/issues/216

        let nix_info = in_github_log_group("info", self.github_output, || async {
            tracing::info!("{}", "\n👟 Gathering NixInfo".bold());
            NixInfo::get()
                .await
                .as_ref()
                .with_context(|| "Unable to gather nix info")
        })
        .await?;

        // First, run the necessary health checks
        in_github_log_group("health", self.github_output, || async {
            tracing::info!("{}", "\n🫀 Performing health check".bold());
            // check_nix_version(&cfg, nix_info).await?;
            check_nix_version(&cfg, nix_info).await
        })
        .await?;

        // Then, do the CI steps
        tracing::info!(
            "{}",
            format!("\n🤖 Running CI for {}", self.flake_ref).bold()
        );
        let res = ci_run(&self.nixcmd, verbose, self, &cfg, &nix_info.nix_config).await?;

        let msg = in_github_log_group::<anyhow::Result<String>, _, _>(
            "outlink",
            self.github_output,
            || async {
                let m_out_link = self.get_out_link();
                let s = serde_json::to_string(&res)?;
                let mut path = tempfile::Builder::new()
                    .prefix("om-ci-results-")
                    .suffix(".json")
                    .tempfile()?;
                path.write_all(s.as_bytes())?;

                let results_path =
                    addstringcontext::addstringcontext(&self.nixcmd, path.path(), m_out_link)
                        .await?;
                println!("{}", results_path.display());

                let msg = format!(
                    "Result available at {:?}{}",
                    results_path.as_path(),
                    m_out_link
                        .map(|p| format!(" and symlinked at {:?}", p))
                        .unwrap_or_default()
                );
                Ok(msg)
            },
        )
        .await?;

        tracing::info!("{}", msg);

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

        if let Some(out_link) = self.out_link.as_ref() {
            args.push("--out-link".to_string());
            args.push(out_link.to_string_lossy().to_string());
        }

        if self.no_link {
            args.push("--no-link".to_string());
        }

        args.push(self.flake_ref.to_string());

        args.extend(self.steps_args.to_cli_args());

        args
    }
}

/// Check that Nix version is not too old.
pub async fn check_nix_version(cfg: &OmConfig, nix_info: &NixInfo) -> anyhow::Result<()> {
    let omnix_health = NixHealth::from_om_config(cfg)?;
    let checks = omnix_health
        .nix_version
        .check(nix_info, Some(&cfg.flake_url));
    let exit_code = NixHealth::print_report_returning_exit_code(&checks, false).await?;

    if exit_code != 0 {
        std::process::exit(exit_code);
    }
    Ok(())
}

/// Run CI for all subflakes
pub async fn ci_run(
    cmd: &NixCmd,
    verbose: bool,
    run_cmd: &RunCommand,
    cfg: &OmConfig,
    nix_config: &NixConfig,
) -> anyhow::Result<RunResult> {
    let mut res = HashMap::new();
    let systems = run_cmd.get_systems(cmd, nix_config).await?;

    let (config, attrs) = cfg.get_sub_config_under::<SubflakesConfig>("ci")?;

    // User's filter by subflake name
    let only_subflake = attrs.first();

    for (subflake_name, subflake) in &config.0 {
        let name = subflake_name.italic();

        if let Some(s) = only_subflake {
            if s != subflake_name {
                tracing::info!("\n🍊 {} {}", name, "skipped (deselected out)".dimmed());
                continue;
            }
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

        let steps_res = in_github_log_group(
            &format!("subflake={}", name),
            run_cmd.github_output,
            || async {
                tracing::info!("\n🍎 {}", name);
                subflake
                    .steps
                    .run(cmd, verbose, run_cmd, &systems, &cfg.flake_url, subflake)
                    .await
            },
        )
        .await?;
        res.insert(subflake_name.clone(), steps_res);
    }

    tracing::info!("\n🥳 Success!");

    Ok(RunResult {
        systems,
        flake: cfg.flake_url.clone(),
        result: res,
    })
}

/// Results of the 'ci run' command
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RunResult {
    /// The systems we are building for
    systems: Vec<System>,
    /// The flake being built
    flake: FlakeUrl,
    /// CI result for each subflake
    result: HashMap<String, StepsResult>,
}

impl RunResult {
    /// Get all store paths mentioned in this type.
    pub fn all_out_paths(&self) -> Vec<StorePath> {
        let mut res = vec![];
        for steps_res in self.result.values() {
            if let Some(build) = steps_res.build_step.as_ref() {
                res.extend(build.devour_flake_output.out_paths.clone());
            }
        }
        res
    }
}
