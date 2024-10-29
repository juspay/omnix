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
}

impl std::fmt::Display for Shell {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let shell_str = match self {
            Shell::Zsh => "zsh",
            Shell::Bash => "bash",
        };
        write!(f, "{}", shell_str)
    }
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
            _ => None,
        }
    }

    /// Get shell dotfiles
    fn get_dotfiles(&self) -> Result<Vec<&'static str>, ShellError> {
        match &self {
            Shell::Zsh => Ok(vec![".zshrc"]),
            Shell::Bash => Ok(vec![".bashrc", ".bash_profile", ".profile"]),
        }
    }
}

impl Checkable for Shell {
    fn check(
        &self,
        _nix_info: &nix_rs::info::NixInfo,
        _flake: Option<&nix_rs::flake::url::FlakeUrl>,
    ) -> Vec<Check> {
        let shell = match Shell::current_shell() {
            Some(shell) => shell,
            None => {
                panic!("Unsupported shell");
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
        ShellError::DotfilesNotFound(err) => CheckResult::Red {
            msg: err.to_string(),
            suggestion:
                "Manage Zsh or Bash shells through https://github.com/juspay/nixos-unified-template"
                    .to_owned(),
        },
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

    let dotfiles = shell.get_dotfiles()?;

    // Iterate over each dotfile and check if it is managed by nix
    for dotfile in dotfiles {
        if !check_dotfile_is_managed_by_nix(&home_dir, dotfile)? {
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
    #[error("Cannot read symlink target of : {0}")]
    DotfilesNotFound(String),
}
