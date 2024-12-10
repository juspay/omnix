//! Functions for running `ci run` on remote machine.

use colored::Colorize;
use nix_rs::{
    command::NixCmd,
    flake::{metadata::FlakeMetadata, url::FlakeUrl},
    store::uri::StoreURI,
};
use omnix_common::config::OmConfig;
use std::{ffi::OsString, os::unix::ffi::OsStringExt, path::PathBuf};
use tokio::process::Command;

use crate::command::run::RunResult;

use super::run::RunCommand;

/// Path to Rust source corresponding to this (running) instance of Omnix
const OMNIX_SOURCE: &str = env!("OMNIX_SOURCE");

/// Like [RunCommand::run] but run on a remote Nix store.
pub async fn run_on_remote_store(
    nixcmd: &NixCmd,
    run_cmd: &RunCommand,
    cfg: &OmConfig,
    store_uri: &StoreURI,
) -> anyhow::Result<()> {
    tracing::info!(
        "{}",
        format!("\n🛜 Running CI remotely on {}", store_uri).bold()
    );

    let (local_flake_path, local_flake_url) = cache_flake(nixcmd, cfg).await?;
    let omnix_source = PathBuf::from(OMNIX_SOURCE);
    let StoreURI::SSH(ssh_uri) = store_uri;

    // First, copy the flake and omnix source to the remote store, because we will be needing them when running over ssh.
    nix_rs::copy::nix_copy(
        nixcmd,
        nix_rs::copy::NixCopyOptions {
            to: Some(store_uri.clone()),
            no_check_sigs: true,
            ..Default::default()
        },
        &[&omnix_source, &local_flake_path],
    )
    .await?;

    // If the user requested creation of `om.json`, we copy all built store paths back, so that the resultant om.json available locally contains valid paths. `-o` can thus be used to trick omnix into copying build results back to local store.
    if let Some(results_file) = run_cmd.results.as_ref() {
        // Create a temp file to hold om.json
        let om_json_path = path_from_bytes(
            &run_ssh_with_output(
                &ssh_uri.to_string(),
                &[
                    "nix",
                    "shell",
                    "nixpkgs#coreutils",
                    "-c",
                    "mktemp",
                    "-t",
                    "om.json.XXXXXX",
                ],
            )
            .await?,
        );

        // Then, SSH and run the same `om ci run` CLI but without the `--on` argument.
        run_ssh(
            &ssh_uri.to_string(),
            &om_cli_with(&RunCommand {
                on: None,
                flake_ref: local_flake_url.clone().into(),
                results: Some(om_json_path.clone()),
                ..run_cmd.clone()
            }),
        )
        .await?;

        // Get om.json
        let om_result: RunResult = serde_json::from_slice(
            &run_ssh_with_output(
                &ssh_uri.to_string(),
                &["cat", om_json_path.to_string_lossy().as_ref()],
            )
            .await?,
        )?;

        // Copy the results back to local store
        tracing::info!("{}", "📦 Copying built paths back to local store".bold());
        nix_rs::copy::nix_copy(
            nixcmd,
            nix_rs::copy::NixCopyOptions {
                from: Some(store_uri.clone()),
                no_check_sigs: true,
                ..Default::default()
            },
            om_result.all_out_paths(),
        )
        .await?;

        // Write the om.json to the requested file
        serde_json::to_writer(std::fs::File::create(results_file)?, &om_result)?;
        tracing::info!(
            "Results written to {}",
            results_file.to_string_lossy().bold()
        );
    } else {
        // Then, SSH and run the same `om ci run` CLI but without the `--on` argument.
        run_ssh(
            &ssh_uri.to_string(),
            &om_cli_with(&RunCommand {
                on: None,
                flake_ref: local_flake_url.clone().into(),
                results: None,
                ..run_cmd.clone()
            }),
        )
        .await?;
    }
    Ok(())
}

fn path_from_bytes(bytes: &[u8]) -> PathBuf {
    PathBuf::from(OsString::from_vec(bytes.to_vec()))
}

/// Return the locally cached [FlakeUrl] for the given flake url that points to same selected [ConfigRef].
async fn cache_flake(nixcmd: &NixCmd, cfg: &OmConfig) -> anyhow::Result<(PathBuf, FlakeUrl)> {
    let metadata = FlakeMetadata::from_nix(nixcmd, &cfg.flake_url).await?;
    let path = metadata.path.to_string_lossy().into_owned();
    let attr = cfg.reference.join(".");
    let local_flake_url = if !attr.is_empty() {
        FlakeUrl(path).with_attr(&attr)
    } else {
        FlakeUrl(path)
    };
    Ok((metadata.path, local_flake_url))
}

/// Construct a `nix run ...` based CLI that runs Omnix using given arguments.
///
/// Omnix itself will be compiled from source ([OMNIX_SOURCE]) if necessary. Thus, this invocation is totally independent and can be run on remote machines, as long as the paths exista on the nix store.
fn om_cli_with(run_cmd: &RunCommand) -> Vec<String> {
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
    args.extend(run_cmd.to_cli_args());
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

/// Run SSH command with given arguments and return the stdout.
async fn run_ssh_with_output<I, S>(host: &str, args: I) -> anyhow::Result<Vec<u8>>
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    let mut cmd = Command::new("ssh");
    cmd.args([host, &shell_words::join(args)]);

    nix_rs::command::trace_cmd_with("🐌", &cmd);

    let output = cmd.output().await?;
    if output.status.success() {
        Ok(output.stdout)
    } else {
        Err(anyhow::anyhow!(
            "SSH command failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ))
    }
}
