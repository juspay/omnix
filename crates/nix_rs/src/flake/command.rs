//! Nix commands for working with flakes
use std::{
    collections::{BTreeMap, HashMap},
    path::PathBuf,
};

use nonempty::NonEmpty;
use serde::{Deserialize, Serialize};
use tokio::process::Command;

use crate::command::{CommandError, NixCmd, NixCmdError};

use super::url::FlakeUrl;

/// Run `nix run` on the given flake app.
pub async fn run(
    nixcmd: &NixCmd,
    opts: &FlakeOptions,
    url: &FlakeUrl,
    args: Vec<String>,
) -> Result<(), CommandError> {
    nixcmd
        .run_with(|cmd| {
            cmd.args(["run".to_string()]);
            opts.use_in_command(cmd);
            cmd.args([url.to_string(), "--".to_string()]);
            cmd.args(args);
        })
        .await?;
    Ok(())
}

/// Run `nix develop` on the given flake devshell.
pub async fn develop(
    nixcmd: &NixCmd,
    opts: &FlakeOptions,
    url: &FlakeUrl,
    command: NonEmpty<String>,
) -> Result<(), CommandError> {
    nixcmd
        .run_with(|cmd| {
            opts.use_in_command(cmd);
            cmd.args(["develop".to_string(), url.to_string(), "-c".to_string()]);
            cmd.args(command);
        })
        .await?;
    Ok(())
}

/// Run `nix build`
pub async fn build(
    cmd: &NixCmd,
    opts: &FlakeOptions,
    url: FlakeUrl,
) -> Result<Vec<OutPath>, NixCmdError> {
    let stdout: Vec<u8> = cmd
        .run_with_returning_stdout(|c| {
            opts.use_in_command(c);
            c.args(["build", "--no-link", "--json", &url]);
        })
        .await?;
    let v = serde_json::from_slice::<Vec<OutPath>>(&stdout)?;
    Ok(v)
}

/// Run `nix flake lock`
pub async fn lock(
    cmd: &NixCmd,
    opts: &FlakeOptions,
    args: &[&str],
    url: &FlakeUrl,
) -> Result<(), NixCmdError> {
    cmd.run_with(|c| {
        opts.use_in_command(c);
        c.args(["flake", "lock", url]);
        c.args(args);
    })
    .await?;
    Ok(())
}

/// A path built by nix, as returned by --print-out-paths
#[derive(Serialize, Deserialize)]
pub struct OutPath {
    /// The derivation that built these outputs
    #[serde(rename = "drvPath")]
    pub drv_path: PathBuf,
    /// Build outputs
    pub outputs: HashMap<String, PathBuf>,
}

impl OutPath {
    /// Return the first build output, if any
    pub fn first_output(&self) -> Option<&PathBuf> {
        self.outputs.values().next()
    }
}

/// Nix CLI options when interacting with a flake
#[derive(Debug, Clone)]
pub struct FlakeOptions {
    /// The --override-input option to pass to Nix
    pub override_inputs: BTreeMap<String, FlakeUrl>,

    /// Pass --no-write-lock-file
    pub no_write_lock_file: bool,

    /// Pass --refresh
    pub refresh: bool,

    /// Accept `nixConfig` configuration in flake.nix
    pub accept_flake_config: bool,

    /// The directory from which to run our nix command (such that relative flake URLs resolve properly)
    pub current_dir: Option<PathBuf>,
}

impl Default for FlakeOptions {
    fn default() -> Self {
        Self {
            override_inputs: BTreeMap::new(),
            no_write_lock_file: false,
            refresh: false,
            accept_flake_config: true, // --accept-flake-config is the default
            current_dir: None,
        }
    }
}

impl FlakeOptions {
    /// Apply these options to a (Nix) [Command]
    pub fn use_in_command(&self, cmd: &mut Command) {
        if let Some(curent_dir) = &self.current_dir {
            cmd.current_dir(curent_dir);
        }
        for (name, url) in self.override_inputs.iter() {
            cmd.arg("--override-input").arg(name).arg(url.to_string());
        }
        if self.no_write_lock_file {
            cmd.arg("--no-write-lock-file");
        }
        if self.refresh {
            cmd.arg("--refresh");
        }
        if self.accept_flake_config {
            cmd.arg("--accept-flake-config");
        }
    }
}
