//! Rust wrapper for `nix-store`
use std::path::{Path, PathBuf};

use crate::command::{CommandError, NixCmdError};
use serde::{Deserialize, Serialize};
use tempfile::TempDir;
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

        let stdout = run_awaiting_stdout(&mut cmd).await?;
        let drv_paths: Vec<PathBuf> = String::from_utf8(stdout)?
            .lines()
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
        let mut cmd = self.command();
        cmd.args(["--query", "--requisites", "--include-outputs"])
            .args(drv_paths);

        let stdout = run_awaiting_stdout(&mut cmd).await?;
        Ok(String::from_utf8(stdout)?
            .lines()
            .map(|line| StorePath::new(PathBuf::from(line)))
            .collect())
    }

    /// Create a file in the Nix store such that it escapes garbage collection.
    ///
    /// Return the nix store path added.
    pub async fn add_file_permanently(
        &self,
        symlink: &Path,
        contents: &str,
    ) -> Result<StorePath, NixStoreCmdError> {
        let temp_dir = TempDir::with_prefix("omnix-ci-")?;
        let temp_file = temp_dir.path().join("om.json");
        std::fs::write(&temp_file, contents)?;

        let path = self.nix_store_add(&temp_file).await?;
        self.nix_store_add_root(symlink, &[&path]).await?;
        Ok(path)
    }

    /// Run `nix-store --add` on the give path and return the store path added.
    pub async fn nix_store_add(&self, path: &Path) -> Result<StorePath, NixStoreCmdError> {
        let mut cmd = self.command();
        cmd.arg("--add");

        // nix-store is unable to accept absolute paths if it involves a symlink
        // https://github.com/juspay/omnix/issues/363
        // To workaround this, we pass the file directly.
        if let Some(parent) = path.parent() {
            cmd.current_dir(parent);
            cmd.arg(path.file_name().unwrap());
        } else {
            cmd.arg(path);
        }

        let stdout = run_awaiting_stdout(&mut cmd).await?;
        Ok(StorePath::new(PathBuf::from(
            String::from_utf8(stdout)?.trim_end(),
        )))
    }

    /// Run `nix-store --add-root` on the given paths and return the store path added.
    pub async fn nix_store_add_root(
        &self,
        symlink: &Path,
        paths: &[&StorePath],
    ) -> Result<(), NixStoreCmdError> {
        let mut cmd = self.command();
        cmd.arg("--add-root")
            .arg(symlink)
            .arg("--realise")
            .args(paths);

        run_awaiting_stdout(&mut cmd).await?;
        Ok(())
    }
}

async fn run_awaiting_stdout(cmd: &mut Command) -> Result<Vec<u8>, NixStoreCmdError> {
    crate::command::trace_cmd(cmd);
    let out = cmd.output().await?;
    if out.status.success() {
        Ok(out.stdout)
    } else {
        let stderr = String::from_utf8_lossy(&out.stderr).to_string();
        let exit_code = out.status.code();
        Err(CommandError::ProcessFailed { stderr, exit_code }.into())
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
