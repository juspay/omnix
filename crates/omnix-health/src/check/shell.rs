use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::traits::{Check, CheckResult, Checkable};

#[derive(Debug, Serialize, Deserialize, Clone, Hash, Eq, PartialEq)]
pub struct ShellCheck {
    pub(crate) enable: bool,
    /// Whether to produce [Check::required] checks
    pub(crate) required: bool,
}

impl Default for ShellCheck {
    fn default() -> Self {
        Self {
            enable: true,
            required: false,
        }
    }
}

/// An Unix shell
#[derive(Debug, Serialize, Deserialize, Clone, Hash, Eq, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Shell {
    Zsh,
    Bash,
}

impl Shell {
    /// Returns the user's current [Shell]
    fn current_shell() -> Option<Self> {
        let shell_path =
            PathBuf::from(std::env::var("SHELL").expect("Environment variable `SHELL` not set"));
        Self::from_path(shell_path)
    }

    /// Lookup [Shell] from the given executable path
    /// For example if path is `/bin/zsh`, it would return `Zsh`
    fn from_path(exe_path: PathBuf) -> Option<Self> {
        let shell_name = exe_path
            .file_name()
            .expect("Path does not have a file name component")
            .to_string_lossy();

        match shell_name.as_ref() {
            "zsh" => Some(Shell::Zsh),
            "bash" => Some(Shell::Bash),
            _ => {
                tracing::warn!("Unrecognized shell: {:?}", exe_path);
                None
            }
        }
    }

    /// Get shell dotfiles
    fn get_dotfiles(&self) -> Vec<&'static str> {
        match &self {
            Shell::Zsh => vec![".zshrc", ".zshenv"],
            Shell::Bash => vec![".bashrc", ".bash_profile", ".profile"],
        }
    }
}

impl Checkable for ShellCheck {
    fn check(
        &self,
        _nix_info: &nix_rs::info::NixInfo,
        _flake: Option<&nix_rs::flake::url::FlakeUrl>,
    ) -> Vec<Check> {
        let shell = match Shell::current_shell() {
            Some(shell) => shell,
            None => {
                panic!("Unsupported shell. Please file an issue at <https://github.com/juspay/omnix/issues>");
            }
        };
        let check = Check {
            title: "Shell Configurations".to_string(),
            info: "Dotfiles managed by Nix".to_string(),
            result: check_shell_configuration(shell),
            required: false,
        };
        vec![check]
    }
}

/// Checks configurations of a [Shell] through dotfiles
fn check_shell_configuration(shell: Shell) -> CheckResult {
    match are_dotfiles_nix_managed(&shell) {
        true => CheckResult::Green,
        false => CheckResult::Red {
            msg: format!("Default Shell: {:?} is not managed by Nix", shell),
            suggestion: "You can use `home-manager` to manage shell configuration. See <https://github.com/juspay/nixos-unified-template>".to_string(),
        },
    }
}

/// Checks if all dotfiles for a given [Shell] are managed by nix
///
/// # Returns
/// * `true` if all dotfiles are nix-managed
/// * `false` if any dotfile is not nix-managed
/// * `Err` if there was an error during the check
fn are_dotfiles_nix_managed(shell: &Shell) -> bool {
    let home_dir =
        PathBuf::from(std::env::var("HOME").expect("Environment variable `HOME` not set"));

    // Iterate over each dotfile and check if it is managed by nix
    let mut managed = vec![];
    for dotfile in shell.get_dotfiles() {
        let path = home_dir.join(dotfile);
        if path.exists() {
            match std::fs::read_link(path) {
                Ok(target) => {
                    managed.push(super::direnv::is_path_in_nix_store(&target));
                },
                Err(err) => {
                    tracing::warn!("Dotfile {:?} error: {:?}", dotfile, err);
                }
            }
        } 
    }
    // If all is true, return true
    managed.iter().all(|&x| x) 
        // Some dotfile must exist
        && !managed.is_empty()
}
