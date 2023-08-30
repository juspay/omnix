//! Nix command configuration

use serde::{Deserialize, Serialize};
#[cfg(feature = "ssr")]
use tokio::process::Command;

/// The `nix` command along with its global options.
///
/// See [available global
/// options](https://nixos.org/manual/nix/stable/command-ref/new-cli/nix#options)
pub struct NixCmd {
    pub extra_experimental_features: Vec<String>,
    pub refresh: Refresh,
}

/// Whether to refresh the flake
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
