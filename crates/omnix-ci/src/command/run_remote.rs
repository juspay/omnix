//! Functions for running `ci run` on remote machine.

use colored::Colorize;
use nix_rs::{
    command::{CommandError, NixCmd},
    copy::{nix_copy, NixCopyOptions},
    flake::{metadata::FlakeMetadata, url::FlakeUrl},
    store::{command::NixStoreCmd, path::StorePath, uri::StoreURI},
};
use omnix_common::config::OmConfig;
use std::{
    ffi::{OsStr, OsString},
    fs::File,
    os::unix::ffi::OsStringExt,
    path::{Path, PathBuf},
};
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
    let StoreURI::SSH(ssh_uri, opts) = store_uri;
    tracing::info!(
        "{}",
        format!("\n🛜 Running CI remotely on {} ({:?})", ssh_uri, opts).bold()
    );

    let ((flake_closure, flake_metadata), local_flake_url) = &cache_flake(nixcmd, cfg).await?;
    let omnix_source = PathBuf::from(OMNIX_SOURCE);

    let mut paths_to_push = vec![omnix_source];

    if opts.copy_inputs {
        paths_to_push.push(flake_closure.clone());
    } else {
        paths_to_push.push(flake_metadata.flake.clone());
    }
    // First, copy the flake and omnix source to the remote store, because we will be needing them when running over ssh.
    nix_copy_to_remote(nixcmd, store_uri, &paths_to_push).await?;

    // If out-link is requested, we need to copy the results back to local store - so that when we create the out-link *locally* the paths in it refer to valid paths in the local store. Thus, --out-link can be used to trick Omnix into copying all built paths back.
    if let Some(out_link) = run_cmd.get_out_link() {
        // A temporary location on ssh remote to hold the result
        let tmpdir = parse_path_line(
            &run_ssh_with_output(
                &ssh_uri.to_string(),
                &nixpkgs_cmd("coreutils", &["mktemp", "-d", "-t", "om.json.XXXXXX"]),
            )
            .await?,
        );
        let om_json_path = tmpdir.join("om.json");

        // Then, SSH and run the same `om ci run` CLI but without the `--on` argument but with `--out-link` pointing to the temporary location.
        run_ssh(
            &ssh_uri.to_string(),
            &om_cli_with(
                run_cmd.local_with(local_flake_url.clone().into(), Some(om_json_path.clone())),
            ),
        )
        .await?;

        // Get the out-link store path.
        let om_result_path: StorePath = StorePath::new(parse_path_line(
            &run_ssh_with_output(
                &ssh_uri.to_string(),
                &nixpkgs_cmd(
                    "coreutils",
                    &["readlink", om_json_path.to_string_lossy().as_ref()],
                ),
            )
            .await?,
        ));

        // Copy the results back to local store, including the out-link.
        if opts.copy_outputs {
            tracing::info!("{}", "📦 Copying all results back to local store".bold());
            nix_copy_from_remote(nixcmd, store_uri, &[&om_result_path]).await?;
            let om_results: RunResult = serde_json::from_reader(File::open(&om_result_path)?)?;
            // Copy all paths referenced in results file
            nix_copy_from_remote(nixcmd, store_uri, om_results.all_out_paths()).await?;
        } else {
            tracing::info!(
                "{}",
                "📦 Copying only omnix result back to local store".bold()
            );
            nix_copy_from_remote(nixcmd, store_uri, &[&om_result_path]).await?;
        }

        // Write the local out-link
        let nix_store = NixStoreCmd {};
        nix_store
            .nix_store_add_root(out_link, &[&om_result_path])
            .await?;
        tracing::info!(
            "Results available at {:?} symlinked at {:?}",
            om_result_path.as_path(),
            out_link
        );
    } else {
        // Then, SSH and run the same `om ci run` CLI but without the `--on` argument.
        run_ssh(
            &ssh_uri.to_string(),
            &om_cli_with(run_cmd.local_with(local_flake_url.clone().into(), None)),
        )
        .await?;
    }
    Ok(())
}

async fn nix_copy_to_remote<I, P>(
    nixcmd: &NixCmd,
    store_uri: &StoreURI,
    paths: I,
) -> Result<(), CommandError>
where
    I: IntoIterator<Item = P>,
    P: AsRef<Path> + AsRef<OsStr>,
{
    nix_copy(
        nixcmd,
        NixCopyOptions {
            to: Some(store_uri.to_owned()),
            no_check_sigs: true,
            ..Default::default()
        },
        paths,
    )
    .await
}

async fn nix_copy_from_remote<I, P>(
    nixcmd: &NixCmd,
    store_uri: &StoreURI,
    paths: I,
) -> Result<(), CommandError>
where
    I: IntoIterator<Item = P>,
    P: AsRef<Path> + AsRef<OsStr>,
{
    nix_copy(
        nixcmd,
        NixCopyOptions {
            from: Some(store_uri.to_owned()),
            no_check_sigs: true,
            ..Default::default()
        },
        paths,
    )
    .await
}

fn parse_path_line(bytes: &[u8]) -> PathBuf {
    let trimmed_bytes = bytes.trim_ascii_end();
    PathBuf::from(OsString::from_vec(trimmed_bytes.to_vec()))
}

/// Construct CLI arguments for running a program from nixpkgs using given arguments
fn nixpkgs_cmd(package: &str, cmd: &[&str]) -> Vec<String> {
    let mut args = vec![
        "nix".to_owned(),
        "shell".to_owned(),
        format!("nixpkgs#{}", package),
    ];
    args.push("-c".to_owned());
    args.extend(cmd.iter().map(|s| s.to_string()));
    args
}

/// Return the locally cached [FlakeUrl] for the given flake url that points to same selected [ConfigRef].
async fn cache_flake(
    nixcmd: &NixCmd,
    cfg: &OmConfig,
) -> anyhow::Result<((PathBuf, FlakeMetadata), FlakeUrl)> {
    let metadata = FlakeMetadata::recursive_evaluate(nixcmd, &cfg.flake_url).await?;
    let attr = cfg.reference.join(".");
    let mut local_flake_url = Into::<FlakeUrl>::into(metadata.1.flake.clone());
    if !attr.is_empty() {
        local_flake_url = local_flake_url.with_attr(&attr);
    }
    Ok((metadata, local_flake_url))
}

/// Construct a `nix run ...` based CLI that runs Omnix using given arguments.
///
/// Omnix itself will be compiled from source ([OMNIX_SOURCE]) if necessary. Thus, this invocation is totally independent and can be run on remote machines, as long as the paths exista on the nix store.
fn om_cli_with(run_cmd: RunCommand) -> Vec<String> {
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
