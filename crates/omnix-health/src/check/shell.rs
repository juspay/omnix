use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

use crate::traits::{Check, CheckResult, Checkable};

/// An Unix shell
#[derive(Debug, Default, Serialize, Deserialize, Clone, Hash, Eq, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Shell {
    Zsh,
    #[default]
    Bash,
    /// Unknown shell
    Other(String),
}

impl std::fmt::Display for Shell {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let shell_str = match self {
            Shell::Zsh => "zsh",
            Shell::Bash => "bash",
            Shell::Other(shell) => shell,
        };
        write!(f, "{}", shell_str)
    }
}

impl Shell {
    /// Returns the user's current [Shell]
    fn current_shell() -> Result<Self, ShellError> {
        let shell_path =
            PathBuf::from(std::env::var("SHELL").expect("Environment variable `SHELL` not set"));
        Self::from_path(shell_path)
    }

    /// Lookup [Shell] from the given executable path
    /// For example if path is `/bin/zsh`, it would return `Zsh`
    fn from_path(exe_path: PathBuf) -> Result<Self, ShellError> {
        let shell_name = exe_path
            .file_name()
            .ok_or_else(|| {
                ShellError::InvalidPath("Path does not have a file name component".to_owned())
            })?
            .to_str()
            .ok_or_else(|| ShellError::InvalidPath("File name is not valid UTF-8".to_owned()))?;

        match shell_name {
            "zsh" => Ok(Shell::Zsh),
            "bash" => Ok(Shell::Bash),
            shell => Ok(Shell::Other(shell.to_string())),
        }
    }

    /// Get shell dotfiles
    fn get_dotfiles(shell: &Shell) -> Result<Vec<String>, ShellError> {
        match shell {
            Shell::Zsh => Ok(vec![".zshrc".to_string()]),
            Shell::Bash => Ok(vec![
                ".bashrc".to_string(),
                ".bash_profile".to_string(),
                ".profile".to_string(),
            ]),
            Shell::Other(shell) => Err(ShellError::UnsupportedShell(shell.to_string())),
        }
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
            result: {
                let shell = Shell::current_shell();
                match shell {
                    Ok(shell) => check_shell_configuration(shell),
                    Err(error) => handle_shell_error(error),
                }
            },
            required: false,
        };
        vec![check]
    }
}

/// Checks configurations of a [Shell] through dotfiles
fn check_shell_configuration(shell: Shell) -> CheckResult {
    match are_dotfiles_nix_managed(&shell) {
        Ok(true) => CheckResult::Green,
        Ok(false) => CheckResult::Red {
            msg: format!("Default Shell: {} is not managed by Nix", shell),
            suggestion: format!(
                "Manage {} configurations through https://github.com/juspay/nixos-unified-template",
                shell
            ),
        },
        Err(error) => handle_shell_error(error),
    }
}

// Error handler for the Shell
fn handle_shell_error(error: ShellError) -> CheckResult {
    match error {
        ShellError::UnsupportedShell(err) => CheckResult::Red {
            msg: err.to_string(),
            suggestion: "We support only Bash & Zsh Shells. Manage Zsh or Bash through https://github.com/juspay/nixos-unified-template".to_owned(),
        },
        ShellError::DotfilesNotFound(err) => CheckResult::Red {
            msg: err.to_string(),
            suggestion: "Manage Zsh or Bash shells through https://github.com/juspay/nixos-unified-template".to_owned(),
        },
        error => panic!("Error occurred while checking shell configuration: {}", error),
    }
}

/// Checks if all dotfiles for a given [Shell] are managed by nix
///
/// # Returns
/// * `true` if all dotfiles are nix-managed
/// * `false` if any dotfile is not nix-managed
/// * `Err` if there was an error during the check
fn are_dotfiles_nix_managed(shell: &Shell) -> Result<bool, ShellError> {
    let home_dir =
        PathBuf::from(std::env::var("HOME").expect("Environment variable `HOME` not set"));

    let dotfiles = Shell::get_dotfiles(shell)?;

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
    let target =
        std::fs::read_link(path).map_err(|_| ShellError::DotfilesNotFound(dotfile.to_owned()))?;
    Ok(super::direnv::is_path_in_nix_store(&target))
}

#[derive(thiserror::Error, Debug)]
pub struct DotfilesNotFound(#[from] std::io::Error);

impl std::fmt::Display for DotfilesNotFound {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

#[derive(thiserror::Error, Debug)]
pub enum ShellError {
    #[error("Checking configurations for {0} is not supported")]
    UnsupportedShell(String),

    #[error("Cannot read symlink target of : {0}")]
    DotfilesNotFound(String),

    #[error("Invalid path: {0}")]
    InvalidPath(String),
}
