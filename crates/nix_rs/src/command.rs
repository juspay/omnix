//! Nix base command configuration
//!
//! # Example
//!
//! ```no_run
//! use nix_rs::command::NixCmd;
//! let cmd = NixCmd::default();
//! cmd.run_with_args_returning_stdout(&["--version"]);
//! ```

use std::fmt::{self, Display};

use serde::{Deserialize, Serialize};
use thiserror::Error;
#[cfg(feature = "ssr")]
use tokio::process::Command;
#[cfg(feature = "ssr")]
use tracing::instrument;

/// The `nix` command's global options.
///
/// See [available global
/// options](https://nixos.org/manual/nix/stable/command-ref/new-cli/nix#options)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub struct NixCmd {
    pub extra_experimental_features: Vec<String>,
    pub refresh: Refresh,
}

/// Whether to refresh the flake, by passing --refresh to nix
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub struct Refresh(bool);

impl From<bool> for Refresh {
    fn from(b: bool) -> Self {
        Self(b)
    }
}

impl Default for NixCmd {
    /// The default `nix` command with flakes already enabled.
    fn default() -> Self {
        Self {
            extra_experimental_features: vec!["nix-command".to_string(), "flakes".to_string()],
            refresh: false.into(),
        }
    }
}

#[cfg(feature = "ssr")]
impl NixCmd {
    /// Return a [Command] for this [NixCmd] configuration
    pub fn command(&self) -> Command {
        let mut cmd = Command::new("nix");
        cmd.args(self.args());
        cmd
    }

    /// Run nix with given args, interpreting stdout as JSON, parsing into `T`
    #[cfg(feature = "ssr")]
    pub async fn run_with_args_expecting_json<T>(&self, args: &[&str]) -> Result<T, NixCmdError>
    where
        T: serde::de::DeserializeOwned,
    {
        let stdout = self.run_with_args_returning_stdout(args).await?;
        let v = serde_json::from_slice::<T>(&stdout)?;
        Ok(v)
    }

    /// Run nix with given args, interpreting parsing stdout, via [std::str::FromStr], into `T`
    #[cfg(feature = "ssr")]
    pub async fn run_with_args_expecting_fromstr<T>(&self, args: &[&str]) -> Result<T, NixCmdError>
    where
        T: std::str::FromStr,
        <T as std::str::FromStr>::Err: std::fmt::Display,
    {
        let stdout = self.run_with_args_returning_stdout(args).await?;
        let v = &String::from_utf8_lossy(&stdout);
        let v = T::from_str(v.trim()).map_err(|e| FromStrError(e.to_string()))?;
        Ok(v)
    }

    /// Run nix with given args, returning stdout.
    #[cfg(feature = "ssr")]
    pub async fn run_with_args_returning_stdout(
        &self,
        args: &[&str],
    ) -> Result<Vec<u8>, CommandError> {
        let mut cmd = self.command();
        cmd.args(args);
        run_command(&mut cmd).await
    }

    /// Convert this [NixCmd] configuration into a list of arguments for
    /// [Command]
    fn args(&self) -> Vec<String> {
        let mut args = vec![];
        if !self.extra_experimental_features.is_empty() {
            args.push("--extra-experimental-features".to_string());
            args.push(self.extra_experimental_features.join(" "));
        }
        if self.refresh.0 {
            args.push("--refresh".to_string());
        }
        args
    }
}

/// Errors when running and interpreting the output of a nix command
#[derive(Error, Debug)]
pub enum NixCmdError {
    #[error("Command error: {0}")]
    CmdError(#[from] CommandError),

    #[error("Failed to decode command stdout (utf8 error): {0}")]
    DecodeErrorUtf8(#[from] std::string::FromUtf8Error),

    #[error("Failed to decode command stdout (from_str error): {0}")]
    DecodeErrorFromStr(#[from] FromStrError),

    #[error("Failed to decode command stdout (json error): {0}")]
    DecodeErrorJson(#[from] serde_json::Error),
}

#[derive(Debug)]
pub struct FromStrError(String);

impl Display for FromStrError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Failed to parse string: {}", self.0)
    }
}

impl std::error::Error for FromStrError {}

/// Run the given command, returning its stdout.
#[cfg(feature = "ssr")]
#[allow(clippy::needless_pass_by_ref_mut)]
pub async fn run_command(cmd: &mut tokio::process::Command) -> Result<Vec<u8>, CommandError> {
    cmd.kill_on_drop(true);
    run_command_(cmd).await
}

#[cfg(feature = "ssr")]
#[instrument(name = "run-command", err)]
async fn run_command_(cmd: &mut tokio::process::Command) -> Result<Vec<u8>, CommandError> {
    tracing::info!("️🏃️ Running command");
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
