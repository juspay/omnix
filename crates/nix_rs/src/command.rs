//! Nix base command configuration
//!
//! # Example
//!
//! ```ignore
//! use nix_rs::command::NixCmd;
//! let cmd = NixCmd::default();
//! cmd.run_with_args_returning_stdout(&["--version"]);
//! ```

use std::{
    fmt::{self, Display},
    process::Stdio,
};

use serde::{Deserialize, Serialize};
use thiserror::Error;

use tokio::{process::Command, sync::OnceCell};

use tracing::instrument;

#[cfg(feature = "clap")]
use clap;

use crate::config::NixConfig;

/// The `nix` command's global options.
///
/// See [available global
/// options](https://nixos.org/manual/nix/stable/command-ref/new-cli/nix#options)
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize, Hash)]
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

static NIXCMD: OnceCell<NixCmd> = OnceCell::const_new();

/// Trace a user-copyable command line
///
/// [tracing::info!] the given [tokio::process::Command] with human-readable
/// command-line string that can generally be copy-pasted by the user.
///
/// The command will be highlighted to distinguish it (for copying) from the
/// rest of the instrumentation parameters.
#[instrument(name = "command")]
pub fn trace_cmd(cmd: &tokio::process::Command) {
    trace_cmd_with("❄️ ", cmd);
}

/// Like [trace_cmd] but with a custom icon
#[instrument(name = "command")]
pub fn trace_cmd_with(icon: &str, cmd: &tokio::process::Command) {
    use colored::Colorize;
    tracing::info!("{}", format!("{} {}️", icon, to_cli(cmd)).dimmed());
}

impl NixCmd {
    /// Return a global `NixCmd` instance with flakes enabled.
    pub async fn get() -> &'static NixCmd {
        NIXCMD
            .get_or_init(|| async {
                let cfg = NixConfig::get().await.as_ref().unwrap();
                let mut cmd = NixCmd::default();
                if !cfg.is_flakes_enabled() {
                    cmd.with_flakes()
                }
                cmd
            })
            .await
    }

    /// Enable flakes on this [NixCmd] configuration
    pub fn with_flakes(&mut self) {
        self.extra_experimental_features
            .append(vec!["nix-command".to_string(), "flakes".to_string()].as_mut());
    }

    /// Enable nix-command on this [NixCmd] configuration
    pub fn with_nix_command(&mut self) {
        self.extra_experimental_features
            .append(vec!["nix-command".to_string()].as_mut());
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
        let stdout: Vec<u8> = self
            .run_with_returning_stdout(|c| {
                c.args(args);
            })
            .await?;
        let v = serde_json::from_slice::<T>(&stdout)?;
        Ok(v)
    }

    /// Run nix with given args, interpreting parsing stdout, via [std::str::FromStr], into `T`
    pub async fn run_with_args_expecting_fromstr<T>(&self, args: &[&str]) -> Result<T, NixCmdError>
    where
        T: std::str::FromStr,
        <T as std::str::FromStr>::Err: std::fmt::Display,
    {
        let stdout = self
            .run_with_returning_stdout(|c| {
                c.args(args);
            })
            .await?;
        let v = &String::from_utf8_lossy(&stdout);
        let v = T::from_str(v.trim()).map_err(|e| FromStrError(e.to_string()))?;
        Ok(v)
    }

    /// Like [Self::run_with] but returns stdout as a [`Vec<u8>`]
    pub async fn run_with_returning_stdout<F>(&self, f: F) -> Result<Vec<u8>, CommandError>
    where
        F: FnOnce(&mut Command),
    {
        let mut cmd = self.command();
        f(&mut cmd);
        trace_cmd(&cmd);

        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());
        let child = cmd.spawn()?;
        let out = child.wait_with_output().await?;

        if out.status.success() {
            Ok(out.stdout)
        } else {
            let stderr = String::from_utf8_lossy(&out.stderr).to_string();
            Err(CommandError::ProcessFailed {
                stderr,
                exit_code: out.status.code(),
            })
        }
    }

    /// Run Nix with given [Command] customizations, while also tracing the command being run.
    ///
    /// Return the stdout bytes returned by [tokio::process::Child::wait_with_output]. In order to capture stdout, you must call `cmd.stdout(Stdio::piped());` inside the handler.
    pub async fn run_with<F>(&self, f: F) -> Result<Vec<u8>, CommandError>
    where
        F: FnOnce(&mut Command),
    {
        let mut cmd = self.command();
        f(&mut cmd);
        trace_cmd(&cmd);
        let out = cmd.spawn()?.wait_with_output().await?;
        if out.status.success() {
            Ok(out.stdout)
        } else {
            let stderr = String::from_utf8_lossy(&out.stderr).to_string();
            Err(CommandError::ProcessFailed {
                stderr,
                exit_code: out.status.code(),
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
    /// A [CommandError]
    #[error("Command error: {0}")]
    CmdError(#[from] CommandError),

    /// Failed to unicode-decode the output of a command
    #[error("Failed to decode command stdout (utf8 error): {0}")]
    DecodeErrorUtf8(#[from] std::string::FromUtf8Error),

    /// Failed to parse the output of a command
    #[error("Failed to decode command stdout (from_str error): {0}")]
    DecodeErrorFromStr(#[from] FromStrError),

    /// Failed to parse the output of a command as JSON
    #[error("Failed to decode command stdout (json error): {0}")]
    DecodeErrorJson(#[from] serde_json::Error),
}

/// Errors when parsing a string into a type
#[derive(Debug)]
pub struct FromStrError(String);

impl Display for FromStrError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Failed to parse string: {}", self.0)
    }
}

impl std::error::Error for FromStrError {}

/// Errors when running a command
#[derive(Error, Debug)]
pub enum CommandError {
    /// Error when spawning a child process
    #[error("Child process error: {0}")]
    ChildProcessError(#[from] std::io::Error),

    /// Child process exited unsuccessfully
    #[error(
        "Process exited unsuccessfully. exit_code={:?} stderr={}",
        exit_code,
        stderr
    )]
    ProcessFailed {
        /// The stderr of the process, if available.
        stderr: String,
        /// The exit code of the process
        exit_code: Option<i32>,
    },

    /// Failed to decode the stderr of a command
    #[error("Failed to decode command stderr: {0}")]
    Decode(#[from] std::string::FromUtf8Error),
}
