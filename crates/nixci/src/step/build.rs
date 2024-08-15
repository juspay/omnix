use clap::Parser;
use colored::Colorize;
use nix_rs::{command::NixCmd, flake::url::FlakeUrl, store::NixStoreCmd};
use serde::Deserialize;

use crate::{
    command::run::RunCommand,
    config::subflake::SubflakeConfig,
    nix::{self, devour_flake},
};

/// Represents a build step in the CI pipeline
///
/// It builds all flake outputs.
#[derive(Debug, Deserialize)]
pub struct BuildStep {
    pub enable: bool,
}

impl Default for BuildStep {
    fn default() -> Self {
        BuildStep { enable: true }
    }
}

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
    pub fn preprocess(&mut self) {
        // Adjust to devour_flake's expectations
        devour_flake::transform_override_inputs(&mut self.extra_nix_build_args);
    }
}

impl BuildStep {
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
        let nix_args = nix_build_args_for_subflake(subflake, run_cmd, url);
        let output = nix::devour_flake::devour_flake(nixcmd, verbose, nix_args).await?;

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

/// Return the devour-flake `nix build` arguments for building all the outputs in this
/// subflake configuration.
fn nix_build_args_for_subflake(
    subflake: &SubflakeConfig,
    run_cmd: &RunCommand,
    flake_url: &FlakeUrl,
) -> Vec<String> {
    let mut args = vec![flake_url.sub_flake_url(subflake.dir.clone()).0];

    for (k, v) in &subflake.override_inputs {
        args.extend_from_slice(&[
            "--override-input".to_string(),
            format!("flake/{}", k),
            v.0.to_string(),
        ])
    }

    // devour-flake already uses this, so no need to override.
    if run_cmd.systems.0 .0 != "github:nix-systems/empty" {
        args.extend_from_slice(&[
            "--override-input".to_string(),
            "systems".to_string(),
            run_cmd.systems.0 .0.clone(),
        ])
    }

    args.extend(
        run_cmd
            .steps_args
            .build_step_args
            .extra_nix_build_args
            .iter()
            .cloned(),
    );

    args
}
