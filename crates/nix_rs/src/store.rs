/// Rust wrapper for `nix-store`
// TODO: Split this into a package of modules.
use std::{collections::HashSet, fmt, path::PathBuf, str::FromStr};

use crate::command::{CommandError, NixCmdError};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio::process::Command;

/// Represents a path in the Nix store, see: <https://zero-to-nix.com/concepts/nix-store#store-paths>
#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Clone, Hash)]
pub enum StorePath {
    /// Derivation path (ends with `.drv`).
    Drv(PathBuf),
    /// Other paths in the Nix store, such as build outputs.
    /// This won't be a derivation path.
    Other(PathBuf),
}

impl StorePath {
    pub fn new(path: PathBuf) -> Self {
        if path.ends_with(".drv") {
            StorePath::Drv(path)
        } else {
            StorePath::Other(path)
        }
    }

    pub fn as_path(&self) -> &PathBuf {
        match self {
            StorePath::Drv(p) => p,
            StorePath::Other(p) => p,
        }
    }
}

impl fmt::Display for StorePath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_path().display())
    }
}

/// Nix Store URI
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum StoreURI {
    /// Remote SSH store
    SSH(SSHStoreURI),
}

/// Remote SSH store URI
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SSHStoreURI {
    /// SSH user
    pub user: Option<String>,
    /// SSH host
    pub host: String,
}

#[derive(Error, Debug)]
pub enum StoreURIParseError {
    #[error("Invalid URI format")]
    InvalidFormat,
    #[error("Unsupported scheme: {0}")]
    UnsupportedScheme(String),
    #[error("Missing host")]
    MissingHost,
}

impl StoreURI {
    pub fn parse(uri: &str) -> Result<Self, StoreURIParseError> {
        let (scheme, rest) = uri
            .split_once("://")
            .ok_or(StoreURIParseError::InvalidFormat)?;

        match scheme {
            "ssh" => {
                let (user, host) = rest
                    .split_once('@')
                    .map(|(u, h)| (Some(u.to_string()), h))
                    .unwrap_or((None, rest));

                if host.is_empty() {
                    return Err(StoreURIParseError::MissingHost);
                }

                Ok(StoreURI::SSH(SSHStoreURI {
                    user,
                    host: host.to_string(),
                }))
            }
            // Add future schemes here
            _ => Err(StoreURIParseError::UnsupportedScheme(scheme.to_string())),
        }
    }
}

impl FromStr for StoreURI {
    type Err = StoreURIParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        StoreURI::parse(s)
    }
}

impl fmt::Display for SSHStoreURI {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(user) = &self.user {
            write!(f, "{}@{}", user, self.host)
        } else {
            write!(f, "{}", self.host)
        }
    }
}
impl fmt::Display for StoreURI {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StoreURI::SSH(uri) => {
                write!(f, "ssh://{}", uri)
            }
        }
    }
}

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
