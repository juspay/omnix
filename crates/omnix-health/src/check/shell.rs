use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

use crate::traits::{Check, CheckResult, Checkable};

/// Shell types
///
#[derive(Debug, Serialize, Deserialize, Clone, Hash, Eq, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Shell {
    Zsh,
    Bash,
    Undeterminable,
    #[serde(other)]
    Other,
}

impl std::fmt::Display for Shell {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let shell_str = match self {
            Shell::Zsh => "zsh",
            Shell::Bash => "bash",
            Shell::Other => "<unsupported shell>",
            Shell::Undeterminable => "<undeterminable shell>",
        };
        write!(f, "{}", shell_str)
    }
}

impl Shell {
    /// Creates a `Shell` from a path string.
    fn from_path(path: &str) -> Self {
        let shell_name = Path::new(path).file_name().and_then(|name| name.to_str());

        match shell_name {
            None => Shell::Undeterminable,
            Some("zsh") => Shell::Zsh,
            Some("bash") => Shell::Bash,
            Some(_) => Shell::Other,
        }
    }

    /// Returns the dotfiles for a given `shell`
    fn get_dotfiles_of_shell(shell: &Shell) -> Result<Vec<String>, ShellError> {
        match shell {
            Shell::Zsh => Ok(vec![".zshrc".to_string()]),
            Shell::Bash => Ok(vec![
                ".bashrc".to_string(),
                ".bash_profile".to_string(),
                ".profile".to_string(),
            ]),
            Shell::Other => Err(ShellError::UnsupportedShell),
            Shell::Undeterminable => Err(ShellError::UndeterminableShell),
        }
    }
}

impl Default for Shell {
    fn default() -> Self {
        let shell_path = std::env::var("SHELL").expect("Environment variable `SHELL` not set");
        Shell::from_path(&shell_path)
    }
}

impl Checkable for Shell {
    fn check(
        &self,
        _nix_info: &nix_rs::info::NixInfo,
        _flake: Option<&nix_rs::flake::url::FlakeUrl>,
    ) -> Vec<Check> {
        let check = Check {
            title: "Shell Configurations".to_string(),
            info: "Dotfiles managed by Nix".to_string(),
            result: match are_dotfiles_nix_managed(self) {
                Ok(true) => CheckResult::Green,
                Ok(false) => {
                    let shell = Shell::default();
                    CheckResult::Red {
                        msg: format!(
                            "Default Shell: {} is not managed by Nix", 
                            shell
                        ),
                        suggestion: format!("Manage {} configurations through https://github.com/juspay/nixos-unified-template", shell) 
                    }
                }
                Err(ShellError::UnsupportedShell) => CheckResult::Red {
                    msg: "Checking configurations for shell is not supported".to_owned(),
                    suggestion: "We support only Bash & Zsh Shells. Manage Zsh or Bash through https://github.com/juspay/nixos-unified-template".to_owned(),
                },
                Err(error) => {
                    panic!(
                        "Error occurred while checking shell configuration: {}",
                        error
                    );
                }
            },
            required: false,
        };
        vec![check]
    }
}

/// Checks if all dotfiles for a given shell are managed by nix
///
/// # Returns
/// * `true` if all dotfiles are nix-managed
/// * `false` if any dotfile is not nix-managed
/// * `Err` if there was an error during the check
fn are_dotfiles_nix_managed(shell: &Shell) -> Result<bool, ShellError> {
    let home_dir = PathBuf::from(
        std::env::var("HOME").map_err(|_| ShellError::EnvVarNotSet("HOME".to_string()))?,
    );

    let dotfiles = Shell::get_dotfiles_of_shell(shell)?;

    // Iterate over each dotfile and check if it is managed by nix
    for dotfile in dotfiles {
        if !check_dotfile_is_managed_by_nix(&home_dir, &dotfile)? {
            return Ok(false);
        }
    }
    Ok(true)
}

fn check_dotfile_is_managed_by_nix(home_dir: &Path, dotfile: &str) -> Result<bool, ShellError> {
    let path = home_dir.join(dotfile);
    let canonical_path = std::fs::canonicalize(&path).map_err(CanonicalizeError)?;
    Ok(super::direnv::is_path_in_nix_store(&canonical_path))
}

#[derive(thiserror::Error, Debug)]
pub struct CanonicalizeError(#[from] std::io::Error);

impl std::fmt::Display for CanonicalizeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

#[derive(thiserror::Error, Debug)]
pub enum ShellError {
    #[error("Checking configurations for shell is not supported")]
    UnsupportedShell,

    #[error("Unable to determine user's default shell")]
    UndeterminableShell,

    #[error("Cannot canonicalize config path: {0}")]
    CanonicalizeError(#[from] CanonicalizeError),

    #[error("Environent variable {0} not set")]
    EnvVarNotSet(String),
}
