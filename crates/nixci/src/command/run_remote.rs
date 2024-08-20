//! Functions for running `ci run` on remote machine.

use colored::Colorize;
use nix_rs::{
    command::NixCmd,
    flake::{metadata::FlakeMetadata, url::FlakeUrl},
    store::StoreURI,
};
use std::path::PathBuf;
use tokio::process::Command;

use crate::config::ref_::ConfigRef;

use super::run::RunCommand;

/// Path to Rust source corresponding to this (running) instance of Omnix
const OMNIX_SOURCE: &str = env!("OMNIX_SOURCE");

/// Like [RunCommand::run] but run on a remote Nix store.
pub async fn run_on_remote_store(
    nixcmd: &NixCmd,
    run_cmd: &RunCommand,
    cfg_ref: &ConfigRef,
    store_uri: &StoreURI,
) -> anyhow::Result<()> {
    tracing::info!(
        "{}",
        format!("\n🛜 Running CI remotely on {}", store_uri).bold()
    );

    let (local_flake_path, local_flake_url) = cache_flake(nixcmd, cfg_ref).await?;
    let omnix_source = PathBuf::from(OMNIX_SOURCE);

    // First, copy the flake and omnix source to the remote store, because we will be needing them when running over ssh.
    nix_rs::copy::nix_copy(nixcmd, store_uri, &[&omnix_source, &local_flake_path]).await?;

    // Then, SSH and run the same `om ci run` CLI but without the `--on` argument.
    match store_uri {
        StoreURI::SSH(ssh_uri) => {
            run_ssh(
                &ssh_uri.to_string(),
                &om_cli_with(run_cmd, &local_flake_url),
            )
            .await
        }
    }
}

/// Return the locally cached [FlakeUrl] for the given flake url that points to same selected [ConfigRef].
async fn cache_flake(nixcmd: &NixCmd, cfg_ref: &ConfigRef) -> anyhow::Result<(PathBuf, FlakeUrl)> {
    let metadata = FlakeMetadata::from_nix(nixcmd, &cfg_ref.flake_url).await?;
    let path = metadata.path.to_string_lossy().into_owned();
    let local_flake_url = if let Some(attr) = cfg_ref.get_attr().0 {
        FlakeUrl(path).with_attr(&attr)
    } else {
        FlakeUrl(path)
    };
    Ok((metadata.path, local_flake_url))
}

/// Construct a `nix run ...` based CLI that runs Omnix using given arguments.
///
/// Omnix itself will be compiled from source ([OMNIX_SOURCE]) if necessary. Thus, this invocation is totally independent and can be run on remote machines, as long as the paths exista on the nix store.
fn om_cli_with(run_cmd: &RunCommand, flake_url: &FlakeUrl) -> Vec<String> {
    let mut args: Vec<String> = vec![];

    let omnix_flake = format!("{}#default", OMNIX_SOURCE);
    args.extend(
        [
            "nix",
            "--accept-flake-config",
            "run",
            &omnix_flake,
            "--",
            "ci",
            "run",
        ]
        .map(&str::to_owned),
    );

    // Re-add current CLI arguments, with a couple of tweaks:
    args.extend(
        RunCommand {
            // Remove --on
            on: None,
            // Substitute with flake path from the nix store
            flake_ref: flake_url.clone().into(),
            ..run_cmd.clone()
        }
        .to_cli_args(),
    );
    args
}

/// Run SSH command with given arguments.
async fn run_ssh(host: &str, args: &[String]) -> anyhow::Result<()> {
    let mut cmd = Command::new("ssh");
    cmd.args([host, &shell_words::join(args)]);

    nix_rs::command::trace_cmd_with("🐌", &cmd);

    cmd.status()
        .await?
        .exit_ok()
        .map_err(|e| anyhow::anyhow!("SSH command failed: {}", e))
}
