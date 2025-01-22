//! The build step
use clap::Parser;
use colored::Colorize;
use nix_rs::{
    command::NixCmd,
    flake::{functions::core::FlakeFn, url::FlakeUrl},
    store::{command::NixStoreCmd, path::StorePath},
};
use serde::{Deserialize, Serialize};

use crate::{
    command::run::RunCommand,
    config::subflake::SubflakeConfig,
    nix::devour_flake::{DevourFlake, DevourFlakeInput, DevourFlakeOutput},
};

/// Represents a build step in the CI pipeline
///
/// It builds all flake outputs.
#[derive(Debug, Clone, Deserialize)]
pub struct BuildStep {
    /// Whether to enable this step
    pub enable: bool,
}

impl Default for BuildStep {
    fn default() -> Self {
        BuildStep { enable: true }
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
    ) -> anyhow::Result<BuildStepResult> {
        // Run devour-flake to do the actual build.
        tracing::info!(
            "{}",
            format!("⚒️  Building subflake: {}", subflake.dir).bold()
        );
        let nix_args = subflake_extra_args(subflake, &run_cmd.steps_args.build_step_args);
        let output = DevourFlake::call(
            nixcmd,
            verbose,
            false,
            None,
            None,
            nix_args,
            DevourFlakeInput {
                flake: url.sub_flake_url(subflake.dir.clone()),
                systems: run_cmd.systems.clone().map(|l| l.0),
            },
        )
        .await?
        .1;

        let mut res = BuildStepResult {
            devour_flake_output: output,
            all_deps: None,
        };

        if run_cmd.steps_args.build_step_args.print_all_dependencies {
            // Handle --print-all-dependencies
            let all_paths = NixStoreCmd
                .fetch_all_deps(&res.devour_flake_output.out_paths)
                .await?;
            res.all_deps = Some(all_paths);
        }

        Ok(res)
    }
}

/// Extra args to pass to devour-flake
fn subflake_extra_args(subflake: &SubflakeConfig, build_step_args: &BuildStepArgs) -> Vec<String> {
    let mut args = vec![];

    for (k, v) in &subflake.override_inputs {
        args.extend([
            "--override-input".to_string(),
            k.to_string(),
            v.0.to_string(),
        ])
    }

    args.extend(build_step_args.extra_nix_build_args.iter().cloned());

    args
}

/// CLI arguments for [BuildStep]
#[derive(Parser, Debug, Clone)]
pub struct BuildStepArgs {
    /// Print build and runtime dependencies along with out paths
    ///
    /// By default, `om ci run` prints only the out paths. This option is
    /// useful to explicitly push all dependencies to a cache.
    #[clap(long, short = 'd')]
    pub print_all_dependencies: bool,

    /// Additional arguments to pass through to `nix build`
    #[arg(last = true, default_values_t = vec![
    "--refresh".to_string(),
    "-j".to_string(),
    "auto".to_string(),
    ])]
    pub extra_nix_build_args: Vec<String>,
}

impl BuildStepArgs {
    /// Convert this type back to the user-facing command line arguments
    pub fn to_cli_args(&self) -> Vec<String> {
        let mut args = vec![];

        if self.print_all_dependencies {
            args.push("--print-all-dependencies".to_owned());
        }

        if !self.extra_nix_build_args.is_empty() {
            args.push("--".to_owned());
            for arg in &self.extra_nix_build_args {
                args.push(arg.clone());
            }
        }

        args
    }
}

/// The result of the build step
#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BuildStepResult {
    /// Output of devour-flake
    #[serde(flatten)]
    pub devour_flake_output: DevourFlakeOutput,

    /// All dependencies of the out paths, if available
    #[serde(skip_serializing_if = "Option::is_none", rename = "allDeps")]
    pub all_deps: Option<Vec<StorePath>>,
}

impl BuildStepResult {
    /// Print the result to stdout
    pub fn print(&self) {
        let paths = if let Some(paths) = &self.all_deps {
            paths
        } else {
            &self.devour_flake_output.out_paths
        };
        for path in paths {
            println!("{}", path.as_path().display());
        }
    }
}
