//! Function for working with `nix run on ssh`.
use std::path::{Path, PathBuf};

use crate::{config::ref_::ConfigRef, step::build::BuildStepArgs};

use anyhow::Context;
use tokio::process::Command;

/// Runs `nix run command through ssh on remote machine` in Rust
///
pub async fn on_ssh(
    build_step_args: &BuildStepArgs,
    remote_address: &str,
    omnix_input: &Path,
    flake_url: PathBuf,
    cfg_ref: ConfigRef,
) -> anyhow::Result<()> {
    let mut cmd = Command::new("ssh");

    // Add the remote address
    cmd.arg(remote_address);

    let mut flake_to_build = flake_url.to_string_lossy().as_ref().to_string();

    // add sub-flake if selected to be built
    if let Some(sub_flake) = cfg_ref.selected_subflake {
        flake_to_build.push_str(&format!("#{}.{}", cfg_ref.selected_name, sub_flake).to_string());
    }

    // base nix command
    let mut nix_cmd = vec![
        "nix run".to_string(),
        format!("{}#default", omnix_input.to_string_lossy().as_ref()),
        "--".to_string(),
        "ci".to_string(),
        "run".to_string(),
        flake_to_build.to_string(),
    ];

    // Add print-all-dependencies flag if passed
    if build_step_args.print_all_dependencies {
        nix_cmd.push("--print-all-dependencies".to_string());
    }

    // Add extra nix build arguments
    nix_cmd.push("--".to_string());
    nix_cmd.extend(build_step_args.extra_nix_build_args.iter().cloned());

    // Join all arguments into a single string
    let args = nix_cmd.join(" ");

    // Add the nix command arguments to the base ssh-command
    cmd.arg(args);

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
