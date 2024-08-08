use crate::cli;

use super::nix_store::StorePath;
use anyhow::Context;
use tokio::process::Command;

use cli::BuildConfig;

pub async fn ssh_run_omnix_ci(
    build_cfg: &BuildConfig,
    remote_address: &str,
    omnix_input: &str,
    flake_url: &str,
) -> anyhow::Result<Vec<StorePath>> {
    // Construct the SSH command
    let mut cmd = Command::new("ssh");
    // Add the remote address to ssh command
    cmd.arg(remote_address);
    // Construct the remote command
    let mut remote_cmd = format!("nix run {}#default -- ci build {}", omnix_input, flake_url);

    if build_cfg.print_all_dependencies {
        remote_cmd.push_str(" --print-all-dependencies");
    }

    // Does it need to used ?
    remote_cmd.push_str(" -- ");
    for arg in &build_cfg.extra_nix_build_args {
        remote_cmd.push_str(arg);
        remote_cmd.push_str(" ");
    }

    cmd.arg(remote_cmd);

    nix_rs::command::trace_cmd(&cmd);

    let status = cmd
        .status()
        .await
        .context("Failed to execute SSH command")?;
    if status.success() {
        Ok(Vec::new())
    } else {
        let exit_code = status.code().unwrap_or(1);
        anyhow::bail!("SSH command failed with exit code: {}", exit_code)
    }
}
