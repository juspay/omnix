//! Nix base command configuration
//!
//! # Example
//!
//! ```ignore
//! use nix_rs::command::NixCmd;
//! let cmd = NixCmd::default();
//! cmd.run_with_args_returning_stdout(&["--version"]);
//! ```

use std::fmt::{self, Display};

use serde::{Deserialize, Serialize};
use thiserror::Error;

use tokio::process::Command;

use tracing::instrument;

#[cfg(feature = "clap")]
use clap;

/// The `nix` command's global options.
///
/// See [available global
/// options](https://nixos.org/manual/nix/stable/command-ref/new-cli/nix#options)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Hash)]
#[cfg_attr(feature = "clap", derive(clap::Parser))]
pub struct NixCmd {
    /// Append to the experimental-features setting of Nix.
    #[cfg_attr(feature = "clap", arg(long))]
    pub extra_experimental_features: Vec<String>,

    /// Append to the access-tokens setting of Nix.
    #[cfg_attr(feature = "clap", arg(long))]
    pub extra_access_tokens: Vec<String>,

    /// Consider all previously downloaded files out-of-date.
    #[cfg_attr(feature = "clap", arg(long))]
    pub refresh: bool,
}

impl Default for NixCmd {
    /// The default `nix` command with flakes already enabled.
    ///
    /// See [NixCmd::with_flakes] for a smarter version that adds feature options only when necessary.
    fn default() -> Self {
        Self {
            extra_experimental_features: vec!["nix-command".to_string(), "flakes".to_string()],
            extra_access_tokens: vec![],
            refresh: false,
        }
    }
}

/// Trace a user-copyable command line
///
/// [tracing::info!] the given [tokio::process::Command] with human-readable
/// command-line string that can generally be copy-pasted by the user.
///
/// The command will be highlighted to distinguish it (for copying) from the
/// rest of the instrumentation parameters.

#[instrument(name = "command")]
pub fn trace_cmd(cmd: &tokio::process::Command) {
    use colored::Colorize;
    tracing::info!("🐚 {}️", to_cli(cmd).bright_blue());
}

impl NixCmd {
    /// Return a `NixCmd` with flakes enabled, if not already enabled.
    pub async fn with_flakes(&self) -> Result<Self, NixCmdError> {
        let cfg = super::config::NixConfig::from_nix(&Self::default()).await?;
        let mut cmd = self.clone();
        if !cfg.is_flakes_enabled() {
            cmd.extra_experimental_features
                .append(vec!["nix-command".to_string(), "flakes".to_string()].as_mut());
        }
        Ok(cmd)
    }

    /// Return a [Command] for this [NixCmd] configuration
    pub fn command(&self) -> Command {
        let mut cmd = Command::new("nix");
        cmd.kill_on_drop(true);
        cmd.args(self.args());
        cmd
    }

    /// Run nix with given args, interpreting stdout as JSON, parsing into `T`
    pub async fn run_with_args_expecting_json<T>(&self, args: &[&str]) -> Result<T, NixCmdError>
    where
        T: serde::de::DeserializeOwned,
    {
        let stdout = self.run_with_args_returning_stdout(args).await?;
        let v = serde_json::from_slice::<T>(&stdout)?;
        Ok(v)
    }

    /// Run nix with given args, interpreting parsing stdout, via [std::str::FromStr], into `T`
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
    pub async fn run_with_args_returning_stdout(
        &self,
        args: &[&str],
    ) -> Result<Vec<u8>, CommandError> {
        let mut cmd = self.command();
        cmd.args(args);
        trace_cmd(&cmd);
        let out = cmd.output().await?;
        if out.status.success() {
            Ok(out.stdout)
        } else {
            let stderr = String::from_utf8(out.stderr)?;
            Err(CommandError::ProcessFailed {
                stderr: Some(stderr),
                exit_code: out.status.code(),
            })
        }
    }

    /// Run nix with given args, letting stdout and stderr be that of parent process
    pub async fn run_with_args(&self, args: &[&str]) -> Result<(), CommandError> {
        let mut cmd = self.command();
        cmd.args(args);
        trace_cmd(&cmd);
        let status = cmd.spawn()?.wait().await?;
        if status.success() {
            Ok(())
        } else {
            Err(CommandError::ProcessFailed {
                stderr: None,
                exit_code: status.code(),
            })
        }
    }

    /// Convert this [NixCmd] configuration into a list of arguments for
    /// [Command]
    fn args(&self) -> Vec<String> {
        let mut args = vec![];
        if !self.extra_experimental_features.is_empty() {
            args.push("--extra-experimental-features".to_string());
            args.push(self.extra_experimental_features.join(" "));
        }
        if !self.extra_access_tokens.is_empty() {
            args.push("--extra-access-tokens".to_string());
            args.push(self.extra_access_tokens.join(" "));
        }
        if self.refresh {
            args.push("--refresh".to_string());
        }
        args
    }
}

/// Convert a Command to user-copyable CLI string
fn to_cli(cmd: &tokio::process::Command) -> String {
    use std::ffi::OsStr;
    let program = cmd.as_std().get_program().to_string_lossy().to_string();
    let args = cmd
        .as_std()
        .get_args()
        .collect::<Vec<&OsStr>>()
        .into_iter()
        .map(|s| s.to_string_lossy().to_string())
        .collect::<Vec<String>>();
    let cli = vec![program]
        .into_iter()
        .chain(args)
        .collect::<Vec<String>>();
    shell_words::join(cli)
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

#[derive(Error, Debug)]
pub enum CommandError {
    #[error("Child process error: {0}")]
    ChildProcessError(#[from] std::io::Error),
    #[error(
        "Process exited unsuccessfully. exit_code={:?}{}",
        exit_code,
        match stderr {
            Some(s) => format!(" stderr={}", s),
            None => "".to_string()
        },
    )]
    ProcessFailed {
        stderr: Option<String>,
        exit_code: Option<i32>,
    },
    #[error("Failed to decode command stderr: {0}")]
    Decode(#[from] std::string::FromUtf8Error),
}
