//! Utilities for running commands

use thiserror::Error;
use tokio::process::Command;
use tracing::instrument;

/// Run the given command, returning its stdout.
#[instrument(name = "run-command", err)]
pub async fn run_command(cmd: &mut Command) -> Result<Vec<u8>, CommandError> {
    tracing::info!("Running command");
    let out = cmd.output().await?;
    if out.status.success() {
        Ok(out.stdout)
    } else {
        let stderr = String::from_utf8(out.stderr)?;
        Err(CommandError::ProcessFailed {
            stderr,
            exit_code: out.status.code(),
        })
    }
}

#[derive(Error, Debug)]
pub enum CommandError {
    #[error("Child process error: {0}")]
    ChildProcessError(#[from] std::io::Error),
    #[error(
        "Process exited unsuccessfully. exit_code={:?} stderr={}",
        exit_code,
        stderr
    )]
    ProcessFailed {
        stderr: String,
        exit_code: Option<i32>,
    },
    #[error("Failed to decode command stderr: {0}")]
    Decode(#[from] std::string::FromUtf8Error),
}
