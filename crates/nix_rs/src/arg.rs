//! Nix command's arguments

use serde::{Deserialize, Serialize};

/// All arguments you can pass to the `nix` command
///
/// This struct is clap-friendly for using in your subcommands. The clap options will mirror that of `nix`.
///
/// To convert to `Command` args list, use `into_iter`.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize, Hash)]
#[cfg_attr(feature = "clap", derive(clap::Parser))]
pub struct NixArgs {
    /// Append to the experimental-features setting of Nix.
    #[cfg_attr(feature = "clap", arg(long))]
    pub extra_experimental_features: Vec<String>,

    /// Append to the access-tokens setting of Nix.
    #[cfg_attr(feature = "clap", arg(long))]
    pub extra_access_tokens: Vec<String>,

    /// Consider all previously downloaded files out-of-date.
    #[cfg_attr(feature = "clap", arg(long))]
    pub refresh: bool,

    /// Additional arguments to pass through to `nix`
    #[arg(last = true, default_values_t = vec![
    "-j".to_string(),
    "auto".to_string(),
    ])]
    pub extra_nix_args: Vec<String>,
}

impl IntoIterator for &NixArgs {
    type Item = String;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.to_args().into_iter()
    }
}

impl NixArgs {
    /// Convert this [NixCmd] configuration into a list of arguments for
    /// [Command]
    fn to_args(&self) -> Vec<String> {
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
        args.extend(self.extra_nix_args.clone());
        args
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
}
