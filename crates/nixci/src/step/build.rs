//! The build step
use clap::Parser;
use colored::Colorize;
use nix_rs::{command::NixCmd, flake::url::FlakeUrl, store::NixStoreCmd};
use serde::Deserialize;

use crate::{
    command::run::RunCommand,
    config::subflake::SubflakeConfig,
    nix::{
        self,
        devour_flake::{self, DevourFlakeInput},
    },
};

/// Represents a build step in the CI pipeline
///
/// It builds all flake outputs.
#[derive(Debug, Deserialize)]
pub struct BuildStep {
    /// Whether to enable this step
    pub enable: bool,
}

impl Default for BuildStep {
    fn default() -> Self {
        BuildStep { enable: true }
    }
}

/// CLI arguments for [BuildStep]
#[derive(Parser, Debug, Clone)]
pub struct BuildStepArgs {
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

impl BuildStepArgs {
    /// Preprocess the arguments
    pub fn preprocess(&mut self) {
        // Adjust to devour_flake's expectations
        devour_flake::transform_override_inputs(&mut self.extra_nix_build_args);
    }
}

impl BuildStep {
    /// Run this step
    pub async fn run(
        &self,
        nixcmd: &NixCmd,
        verbose: bool,
        run_cmd: &RunCommand,
        url: &FlakeUrl,
        subflake: &SubflakeConfig,
    ) -> anyhow::Result<()> {
        // Run devour-flake to do the actual build.
        tracing::info!(
            "{}",
            format!("⚒️  Building subflake: {}", subflake.dir).bold()
        );
        let nix_args = subflake_extra_args(subflake, &run_cmd.steps_args.build_step_args);
        let devour_input = DevourFlakeInput {
            flake: url.sub_flake_url(subflake.dir.clone()),
            systems: run_cmd.systems.0.clone(),
        };
        let output =
            nix::devour_flake::devour_flake(nixcmd, verbose, devour_input, nix_args).await?;

        let outs = if run_cmd.steps_args.build_step_args.print_all_dependencies {
            // Handle --print-all-dependencies
            NixStoreCmd.fetch_all_deps(output.0).await
        } else {
            Ok(output.0)
        }?;

        for out in outs {
            println!("{}", out);
        }
        Ok(())
    }
}

/// Extra args to pass to devour-flake
fn subflake_extra_args(subflake: &SubflakeConfig, build_step_args: &BuildStepArgs) -> Vec<String> {
    let mut args = vec![];

    for (k, v) in &subflake.override_inputs {
        args.extend_from_slice(&[
            "--override-input".to_string(),
            format!("flake/{}", k),
            v.0.to_string(),
        ])
    }

    args.extend(build_step_args.extra_nix_build_args.iter().cloned());

    args
}
