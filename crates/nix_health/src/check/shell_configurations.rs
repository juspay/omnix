use nix_rs::info;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

use crate::traits::{Check, CheckResult, Checkable};

#[derive(Debug, Serialize, Deserialize, Clone, Hash, Eq, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Shell {
    Zsh,
    Bash,
    #[serde(other)]
    Unknown,
}

impl Shell {
    fn from_path(path: &str) -> Self {
        let shell_name = Path::new(path)
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("");

        match shell_name {
            "zsh" => Shell::Zsh,
            "bash" => Shell::Bash,
            _ => Shell::Unknown,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct ShellConfigurations {
    dotfiles: HashMap<Shell, Vec<String>>,
}

impl Default for ShellConfigurations {
    fn default() -> Self {
        let mut dotfiles = HashMap::new();
        dotfiles.insert(Shell::Zsh, vec![".zshrc".to_string()]);
        dotfiles.insert(
            Shell::Bash,
            vec![
                ".bashrc".to_string(),
                ".bash_profile".to_string(),
                ".profile".to_string(),
            ],
        );
        Self { dotfiles }
    }
}

impl ShellConfigurations {
    fn check_shell_configuration(&self) -> bool {
        std::env::var("SHELL")
            .ok()
            .map(|shell| {
                let shell_name = Shell::from_path(&shell);
                self.are_shell_dotfiles_nix_managed(&shell_name)
            })
            .unwrap_or(false)
    }

    fn are_shell_dotfiles_nix_managed(&self, shell: &Shell) -> bool {
        dirs::home_dir()
            .map(|home| {
                self.dotfiles.get(shell).map_or(false, |shell_files| {
                    shell_files.iter().all(|dotfile| {
                        std::fs::canonicalize(home.join(dotfile))
                            .map(|canonical_path| canonical_path.starts_with("/nix/store/"))
                            .unwrap_or(false)
                    })
                })
            })
            .unwrap_or(false)
    }
}

impl Checkable for ShellConfigurations {
    fn check(&self, _: &info::NixInfo, _: Option<&nix_rs::flake::url::FlakeUrl>) -> Vec<Check> {
        let check = Check {
            title: "Shell Configurations".to_string(),
            info: "Dotfiles managed by Nix".to_string(),
            result: if self.check_shell_configuration() {
                CheckResult::Green
            } else {
                CheckResult::Red {
                    msg: "Shell Configurations are not managed by Nix".into(),
                    suggestion: "Manage shell (zsh, bash) configurations through https://github.com/juspay/nixos-unified-template".into(),
                }
            },
            required: false,
        };
        vec![check]
    }
}
