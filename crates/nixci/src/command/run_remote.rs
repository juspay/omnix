//! Functions for running `ci run` on remote machine.

use anyhow::{Context, Result};
use nix_rs::{
    command::NixCmd,
    flake::{metadata::FlakeMetadata, url::FlakeUrl},
};
use std::path::PathBuf;

use crate::{config::ref_::ConfigRef, step::build::BuildStepArgs};
use tokio::process::Command;

/// Path to Rust source corresponding to this (running) instance of Omnix
pub const OMNIX_SOURCE: &str = env!("OMNIX_SOURCE");

/// Run the ci run steps on remote
pub async fn run(
    build_step_args: BuildStepArgs,
    nixcmd: &NixCmd,
    cfg_ref: ConfigRef,
    host: &str,
) -> anyhow::Result<()> {
    let metadata = FlakeMetadata::from_nix(nixcmd, &cfg_ref.flake_url).await?;

    let omnix_source = PathBuf::from(OMNIX_SOURCE);

    nix_rs::copy::nix_copy(nixcmd, host, &[omnix_source.clone(), metadata.path.clone()]).await?;

    let nix_run_args = get_nix_run_args(build_step_args, metadata.path, cfg_ref)?;

    // call ci run on remote machine through ssh
    on_ssh(host, &nix_run_args).await?;

    Ok(())
}

/// Returns `nix run` args for running `ci run` on remote machine.
fn get_nix_run_args(
    build_step_args: BuildStepArgs,
    flake_url: PathBuf,
    cfg_ref: ConfigRef,
) -> Result<Vec<String>> {
    let ci_run_args = get_ci_run_args_for_remote(build_step_args, flake_url, cfg_ref)?;

    let nix_run_args: Vec<String> = vec![
        "nix run".to_string(),
        format!("{}#default", OMNIX_SOURCE),
        "--".to_string(),
    ]
    .into_iter()
    .chain(ci_run_args.into_iter())
    .collect();

    Ok(nix_run_args)
}

/// Returns ci run args along with build_step_args
fn get_ci_run_args_for_remote(
    build_step_args: BuildStepArgs,
    flake_url: PathBuf,
    cfg_ref: ConfigRef,
) -> Result<Vec<String>> {
    let mut flake_to_build = flake_url.to_string_lossy().as_ref().to_string();

    // add sub-flake if selected to be built
    if let Some(sub_flake) = cfg_ref.selected_subflake {
        flake_to_build.push_str(&format!("#{}.{}", cfg_ref.selected_name, sub_flake).to_string());
    }

    let mut nix_run_args = vec![
        "ci".to_string(),
        "run".to_string(),
        flake_to_build.to_string(),
    ];

    // Add print-all-dependencies flag if passed
    if build_step_args.print_all_dependencies {
        nix_run_args.push("--print-all-dependencies".to_string());
    }

    // Add extra nix build arguments
    nix_run_args.push("--".to_string());
    nix_run_args.extend(build_step_args.extra_nix_build_args.iter().cloned());

    Ok(nix_run_args)
}

/// Runs `commands through ssh on remote machine` in Rust
pub async fn on_ssh(remote_address: &str, args: &[String]) -> anyhow::Result<()> {
    let mut cmd = Command::new("ssh");

    // Add the remote address
    cmd.arg(remote_address);

    // Join all arguments in a string and add to ssh command.
    cmd.arg(args.join(" "));

    nix_rs::command::trace_cmd(&cmd);

    let status = cmd
        .status()
        .await
        .context("Failed to execute SSH command")?;

    if status.success() {
        Ok(())
    } else {
        let exit_code = status.code().unwrap_or(1);
        anyhow::bail!("SSH command failed with exit code: {}", exit_code)
    }
}
