use std::{collections::BTreeMap, path::PathBuf};

use nonempty::NonEmpty;
use tokio::process::Command;

use crate::command::{CommandError, NixCmd};

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
            opts.use_in_command(cmd);
            cmd.args(["run".to_string(), url.to_string(), "--".to_string()]);
            cmd.args(args);
        })
        .await
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
        .await
}

/// Nix CLI options when interacting with a flake
#[derive(Debug, Clone)]
pub struct FlakeOptions {
    /// The --override-input option to pass to Nix
    pub override_inputs: BTreeMap<String, FlakeUrl>,

    /// The directory from which to run our nix command (such that relative flake URLs resolve properly)
    pub current_dir: Option<PathBuf>,
}

impl FlakeOptions {
    /// Apply these options to a (Nix) [Command]
    fn use_in_command(&self, cmd: &mut Command) {
        if let Some(curent_dir) = &self.current_dir {
            cmd.current_dir(curent_dir);
        }
        for (name, url) in self.override_inputs.iter() {
            cmd.arg("--override-input").arg(name).arg(url.to_string());
        }
    }
}
