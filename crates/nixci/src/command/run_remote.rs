//! Functions for running `ci run` on remote machine.

use anyhow::{Context, Result};
use colored::Colorize;
use nix_rs::{
    command::NixCmd,
    flake::{metadata::FlakeMetadata, url::FlakeUrl},
    store::{SSHStoreURI, StoreURI},
};
use std::path::PathBuf;
use tokio::process::Command;

use crate::{config::ref_::ConfigRef, step::build::BuildStepArgs};

/// Path to Rust source corresponding to this (running) instance of Omnix
const OMNIX_SOURCE: &str = env!("OMNIX_SOURCE");

/// Run the ci run steps on remote
pub async fn run(
    build_step_args: &BuildStepArgs,
    nixcmd: &NixCmd,
    cfg_ref: &ConfigRef,
    store_uri: &StoreURI,
) -> anyhow::Result<()> {
    tracing::info!(
        "{}",
        format!("\n🛜 Running CI remotely on {}", store_uri).bold()
    );

    let (local_flake_path, local_flake_url) =
        cache_flake(nixcmd, &cfg_ref.flake_url, &cfg_ref).await?;
    let omnix_source = PathBuf::from(OMNIX_SOURCE);

    nix_rs::copy::nix_copy(nixcmd, store_uri, &[&omnix_source, &local_flake_path]).await?;

    let nix_run_args = nix_run_om_ci_run_args(build_step_args, local_flake_url)?;

    match store_uri {
        StoreURI::SSH(ssh_uri) => on_ssh(ssh_uri, &nix_run_args).await,
    }
}

/// Return the locally cached [FlakeUrl] for the given flake url that points to same selected [ConfigRef].
async fn cache_flake(
    nixcmd: &NixCmd,
    flake_url: &FlakeUrl,
    cfg_ref: &ConfigRef,
) -> anyhow::Result<(PathBuf, FlakeUrl)> {
    let metadata = FlakeMetadata::from_nix(nixcmd, flake_url).await?;
    let path = metadata.path.to_string_lossy().into_owned();
    let local_flake_url = if let Some(attr) = cfg_ref.get_attr().0 {
        FlakeUrl(path).with_attr(&attr)
    } else {
        FlakeUrl(path)
    };
    Ok((metadata.path, local_flake_url))
}

/// Returns `nix run` args for running `om ci run` on remote machine.
fn nix_run_om_ci_run_args(
    build_step_args: &BuildStepArgs,
    flake_url: FlakeUrl,
) -> Result<Vec<String>> {
    let mut args: Vec<&str> = vec![];

    let omnix_flake = format!("{}#default", OMNIX_SOURCE);
    args.extend(&["nix run", &omnix_flake, "--"]);
    args.extend(&["ci", "run", &flake_url]);

    if build_step_args.print_all_dependencies {
        args.push("--print-all-dependencies");
    }

    // Add extra nix build arguments
    if !build_step_args.extra_nix_build_args.is_empty() {
        args.push("--");
        for arg in &build_step_args.extra_nix_build_args {
            args.push(&arg);
        }
    }

    Ok(args.iter().map(|s| s.to_string()).collect())
}

/// Runs `commands through ssh on remote machine` in Rust
async fn on_ssh(remote_address: &SSHStoreURI, args: &[String]) -> anyhow::Result<()> {
    let mut cmd = Command::new("ssh");

    // Add the remote address
    cmd.arg(remote_address.to_string());

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
