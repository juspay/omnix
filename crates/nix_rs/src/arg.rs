//! Nix command's arguments

use std::collections::HashMap;

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

    /// Additional arguments to pass through to `nix`
    ///
    /// NOTE: Arguments irrelevant to a nix subcommand will automatically be ignored.
    #[cfg_attr(feature = "clap", arg(last = true, default_values_t = vec![
    "-j".to_string(),
    "auto".to_string(),
    ]))]
    pub extra_nix_args: Vec<String>,
}

impl NixArgs {
    /// Convert this [NixCmd] configuration into a list of arguments for
    /// [Command]
    pub fn to_args(&self, subcommands: &[&str]) -> Vec<String> {
        let mut args = vec![];
        if !self.extra_experimental_features.is_empty() {
            args.push("--extra-experimental-features".to_string());
            args.push(self.extra_experimental_features.join(" "));
        }
        if !self.extra_access_tokens.is_empty() {
            args.push("--extra-access-tokens".to_string());
            args.push(self.extra_access_tokens.join(" "));
        }
        let mut extra_nix_args = self.extra_nix_args.clone();
        remove_nonsense_args_when_subcommand(subcommands, &mut extra_nix_args);
        args.extend(extra_nix_args);
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

/// Certain options, like --rebuild, is not supported by all subcommands (e.g.
/// `nix develop`). Remove them here.
fn remove_nonsense_args_when_subcommand(subcommands: &[&str], args: &mut Vec<String>) {
    let unsupported = non_sense_options(subcommands);
    for (option, count) in unsupported {
        remove_arguments(args, option, count);
    }
}

fn non_sense_options<'a>(subcommands: &[&str]) -> HashMap<&'a str, usize> {
    let rebuild = ("--rebuild", 0);
    let override_input = ("--override-input", 2);
    match subcommands {
        ["eval"] => HashMap::from([rebuild, override_input]),
        ["flake", "lock"] => HashMap::from([rebuild, override_input]),
        ["flake", "check"] => HashMap::from([rebuild]),
        ["develop"] => HashMap::from([rebuild]),
        ["run"] => HashMap::from([rebuild]),
        _ => HashMap::new(),
    }
}

fn remove_arguments(vec: &mut Vec<String>, arg: &str, next: usize) {
    let mut i = 0;
    while i < vec.len() {
        if vec[i] == arg && i + next < vec.len() {
            vec.drain(i..i + next + 1);
        } else {
            i += 1;
        }
    }
}
