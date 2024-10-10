use nix_rs::info;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

use crate::traits::{Check, CheckResult, Checkable};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct ShellConfigurations {
    dotfiles: HashMap<String, Vec<String>>,
}

impl Default for ShellConfigurations {
    fn default() -> Self {
        let mut dotfiles = HashMap::new();
        dotfiles.insert("zsh".to_string(), vec![".zshrc".to_string()]);
        dotfiles.insert(
            "bash".to_string(),
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
        match std::env::var("SHELL").ok() {
            Some(shell) => {
                let shell_name = Path::new(&shell)
                    .file_name()
                    .and_then(|name| name.to_str())
                    .unwrap_or("");

                self.are_shell_dotfiles_nix_managed(shell_name)
            }
            None => false,
        }
    }

    fn are_shell_dotfiles_nix_managed(&self, shell: &str) -> bool {
        let home = match dirs::home_dir() {
            Some(path) => path,
            None => return false,
        };

        self.dotfiles.get(shell).map_or(false, |shell_files| {
            shell_files.iter().all(|dotfile| {
                let path = home.join(dotfile);
                std::fs::canonicalize(&path)
                    .map(|canonical_path| canonical_path.starts_with("/nix/store/"))
                    .unwrap_or(false)
            })
        })
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
