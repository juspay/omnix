//! Function for working with `nix commands on ssh`.

use anyhow::Context;
use tokio::process::Command;

/// Runs `commands through ssh on remote machine` in Rust
///
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
