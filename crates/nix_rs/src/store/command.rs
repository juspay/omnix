//! Rust wrapper for `nix-store`
use std::path::PathBuf;

use crate::command::{CommandError, NixCmdError};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio::process::Command;

use super::path::StorePath;

/// The `nix-store` command
/// See documentation for [nix-store](https://nixos.org/manual/nix/stable/command-ref/nix-store.html)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub struct NixStoreCmd;

impl NixStoreCmd {
    /// Get the associated [Command]
    pub fn command(&self) -> Command {
        let mut cmd = Command::new("nix-store");
        cmd.kill_on_drop(true);
        cmd
    }
}

impl NixStoreCmd {
    /// Fetch all build and runtime dependencies of given derivation outputs.
    ///
    /// This is done by querying the deriver of each derivation output
    /// using [NixStoreCmd::nix_store_query_deriver] and then querying all
    /// dependencies of each deriver using
    /// [NixStoreCmd::nix_store_query_requisites_with_outputs].  Finally, all
    /// dependencies of each deriver are collected and returned as
    /// `Vec<StorePath>`.
    pub async fn fetch_all_deps(
        &self,
        out_paths: &[StorePath],
    ) -> Result<Vec<StorePath>, NixStoreCmdError> {
        let all_drvs = self.nix_store_query_deriver(out_paths).await?;
        let all_outs = self
            .nix_store_query_requisites_with_outputs(&all_drvs)
            .await?;
        Ok(all_outs)
    }

    /// Return the derivations used to build the given build output.
    pub async fn nix_store_query_deriver(
        &self,
        out_paths: &[StorePath],
    ) -> Result<Vec<PathBuf>, NixStoreCmdError> {
        let mut cmd = self.command();
        cmd.args(["--query", "--valid-derivers"])
            .args(out_paths.iter().map(StorePath::as_path));

        crate::command::trace_cmd(&cmd);

        let out = cmd.output().await?;
        if out.status.success() {
            let drv_paths: Vec<PathBuf> = String::from_utf8(out.stdout)?
                .lines()
                .map(PathBuf::from)
                .collect();
            if drv_paths.contains(&PathBuf::from("unknown-deriver")) {
                return Err(NixStoreCmdError::UnknownDeriver);
            }
            Ok(drv_paths)
        } else {
            // TODO(refactor): When upstreaming this module to nix-rs, create a
            // nicer and unified way to create `ProcessFailed`
            let stderr = Some(String::from_utf8_lossy(&out.stderr).to_string());
            let exit_code = out.status.code();
            Err(CommandError::ProcessFailed { stderr, exit_code }.into())
        }
    }

    /// Given the derivation paths, this function recursively queries and return all
    /// of its dependencies in the Nix store.
    pub async fn nix_store_query_requisites_with_outputs(
        &self,
        drv_paths: &[PathBuf],
    ) -> Result<Vec<StorePath>, NixStoreCmdError> {
        let mut cmd = self.command();
        cmd.args(["--query", "--requisites", "--include-outputs"])
            .args(drv_paths);

        crate::command::trace_cmd(&cmd);

        let out = cmd.output().await?;
        if out.status.success() {
            Ok(String::from_utf8(out.stdout)?
                .lines()
                .map(|line| StorePath::new(PathBuf::from(line)))
                .collect())
        } else {
            // TODO(refactor): see above
            let stderr = Some(String::from_utf8_lossy(&out.stderr).to_string());
            let exit_code = out.status.code();
            Err(CommandError::ProcessFailed { stderr, exit_code }.into())
        }
    }
}

/// `nix-store` command errors
#[derive(Error, Debug)]
pub enum NixStoreCmdError {
    /// A [NixCmdError]
    #[error(transparent)]
    NixCmdError(#[from] NixCmdError),

    /// nix-store returned "unknown-deriver"
    #[error("Unknown deriver")]
    UnknownDeriver,
}

impl From<std::io::Error> for NixStoreCmdError {
    fn from(err: std::io::Error) -> Self {
        let cmd_error: CommandError = err.into();
        cmd_error.into()
    }
}

impl From<std::string::FromUtf8Error> for NixStoreCmdError {
    fn from(err: std::string::FromUtf8Error) -> Self {
        let cmd_error: CommandError = err.into();
        cmd_error.into()
    }
}

impl From<CommandError> for NixStoreCmdError {
    fn from(err: CommandError) -> Self {
        let nixcmd_error: NixCmdError = err.into();
        nixcmd_error.into()
    }
}
