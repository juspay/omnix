//! Utilities for running commands

use leptos::ServerFnError;
use tokio::process::Command;

/// Run the given command, returning its stdout.
///
/// Failures are wrapped in Leptos [ServerFnError]s.
pub async fn run_command_in_server_fn(cmd: &mut Command) -> Result<Vec<u8>, ServerFnError> {
    tracing::info!("Running: {:?}", cmd);
    let out = cmd.output().await?;
    if out.status.success() {
        Ok(out.stdout)
    } else {
        let stderr = String::from_utf8(out.stderr)
            .map_err(|e| <std::string::FromUtf8Error as Into<ServerFnError>>::into(e))?;
        let err = ServerFnError::ServerError(format!("Command failed: {}", stderr).into());
        tracing::error!("{}", err);
        Err(err)
    }
}
