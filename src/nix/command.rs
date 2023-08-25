//! Nix command configuration

use tokio::process::Command;

/// The `nix` command along with its global options.
///
/// See [available global
/// options](https://nixos.org/manual/nix/stable/command-ref/new-cli/nix#options)
pub struct NixCmd {
    pub extra_experimental_features: Vec<String>,
    pub refresh: bool,
}

impl Default for NixCmd {
    /// The default `nix` command with flakes already enabled.
    fn default() -> Self {
        Self {
            extra_experimental_features: vec!["nix-command".to_string(), "flakes".to_string()],
            refresh: false,
        }
    }
}

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
        if self.refresh {
            args.push("--refresh".to_string());
        }
        args
    }
}
