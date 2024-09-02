/// Rust wrapper for `nix-store`
use std::{collections::HashSet, path::PathBuf, process::Stdio};

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
        out_paths: HashSet<StorePath>,
    ) -> Result<HashSet<StorePath>, NixStoreCmdError> {
        let all_drvs = self
            .nix_store_query_deriver(&out_paths.iter().cloned().collect::<Vec<_>>())
            .await?;
        let all_outs = self
            .nix_store_query_requisites_with_outputs(&all_drvs)
            .await?;
        Ok(all_outs.into_iter().collect())
    }

    async fn run_query(
        &self,
        args: &[&str],
        paths: &[impl AsRef<std::path::Path>],
    ) -> Result<Vec<String>, NixStoreCmdError> {
        let mut cmd = self.command();
        cmd.args(args)
            .args(paths.iter().map(AsRef::as_ref))
            .stdout(Stdio::piped());
        crate::command::trace_cmd(&cmd);

        let output = cmd.output().await?;
        if !output.status.success() {
            return Err(CommandError::ProcessFailed {
                exit_code: output.status.code(),
            }
            .into());
        }

        Ok(String::from_utf8(output.stdout)?
            .lines()
            .map(String::from)
            .collect())
    }

    /// Return the derivations used to build the given build output.
    pub async fn nix_store_query_deriver(
        &self,
        out_paths: &[StorePath],
    ) -> Result<Vec<PathBuf>, NixStoreCmdError> {
        let drv_paths: Vec<PathBuf> = self
            .run_query(&["--query", "--valid-derivers"], out_paths)
            .await?
            .into_iter()
            .map(PathBuf::from)
            .collect();

        if drv_paths.contains(&PathBuf::from("unknown-deriver")) {
            return Err(NixStoreCmdError::UnknownDeriver);
        }
        Ok(drv_paths)
    }

    /// Given the derivation paths, this function recursively queries and return all
    /// of its dependencies in the Nix store.
    pub async fn nix_store_query_requisites_with_outputs(
        &self,
        drv_paths: &[PathBuf],
    ) -> Result<Vec<StorePath>, NixStoreCmdError> {
        Ok(self
            .run_query(&["--query", "--requisites", "--include-outputs"], drv_paths)
            .await?
            .into_iter()
            .map(|line| StorePath::new(PathBuf::from(line)))
            .collect())
    }
}

/// `nix-store` command errors
#[derive(Error, Debug)]
pub enum NixStoreCmdError {
    #[error(transparent)]
    NixCmdError(#[from] NixCmdError),

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
