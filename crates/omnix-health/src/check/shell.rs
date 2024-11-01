use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::PathBuf};

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

impl Checkable for ShellCheck {
    fn check(
        &self,
        _nix_info: &nix_rs::info::NixInfo,
        _flake: Option<&nix_rs::flake::url::FlakeUrl>,
    ) -> Vec<Check> {
        if !self.enable {
            return vec![];
        }
        let shell = match Shell::current_shell() {
            Some(shell) => shell,
            None => {
                let msg = "Unsupported shell. Please file an issue at <https://github.com/juspay/omnix/issues>";
                if self.required {
                    panic!("{}", msg);
                } else {
                    tracing::warn!("Skipping shell dotfile check! {}", msg);
                    return vec![];
                }
            }
        };

        // Iterate over each dotfile and check if it is managed by nix
        let mut managed: HashMap<PathBuf, PathBuf> = HashMap::new();
        let mut unmanaged: Vec<PathBuf> = Vec::new();
        for path in &shell.get_dotfiles() {
            match std::fs::read_link(path) {
                Ok(target) => {
                    if super::direnv::is_path_in_nix_store(&target) {
                        managed.insert(path.clone(), target);
                    } else {
                        unmanaged.push(path.clone());
                    };
                }
                Err(err) => {
                    tracing::warn!("Dotfile {:?} symlink error: {:?}; ignoring.", path, err);
                }
            }
        }

        let title = "Shell dotfiles".to_string();
        let info = format!("Managed: {:?}; Unmanaged: {:?}", managed, unmanaged);
        let result = if !managed.is_empty() {
            CheckResult::Green
        } else {
            CheckResult::Red {
                msg: format!("Default Shell: {:?} is not managed by Nix", shell),
                    suggestion: "You can use `home-manager` to manage shell configuration. See <https://github.com/juspay/nixos-unified-template>".to_string(),
            }
        };
        let check = Check {
            title,
            info,
            result,
            required: self.required,
        };

        vec![check]
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
                tracing::warn!("Unrecognized shell: {:?}. Please file an issue at <https://github.com/juspay/omnix/issues>", exe_path);
                None
            }
        }
    }

    /// Get shell dotfiles
    fn dotfile_names(&self) -> Vec<&'static str> {
        match &self {
            Shell::Zsh => vec![".zshrc", ".zshenv", ".zprofile"],
            Shell::Bash => vec![".bashrc", ".bash_profile", ".profile"],
        }
    }

    /// Get the currently existing dotfiles under $HOME
    fn get_dotfiles(&self) -> Vec<PathBuf> {
        let home_dir =
            PathBuf::from(std::env::var("HOME").expect("Environment variable `HOME` not set"));
        self.dotfile_names()
            .iter()
            .map(|dotfile| home_dir.join(dotfile))
            .filter(|path| path.exists())
            .collect()
    }
}
