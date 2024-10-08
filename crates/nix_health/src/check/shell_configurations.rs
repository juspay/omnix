use nix_rs::info;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, env, path::PathBuf};

use crate::traits::{Check, CheckResult, Checkable};

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct ShellConfigurations {}

impl Checkable for ShellConfigurations {
    fn check(&self, _: &info::NixInfo, _: Option<&nix_rs::flake::url::FlakeUrl>) -> Vec<Check> {
        let check = Check {
            title: "Shell Configurations".to_string(),
            info: "Shell Configurations are managed by Nix".to_string(),
            result: if check_shell_configuration() {
                CheckResult::Green
            } else {
                CheckResult::Red {
                    msg: "Shell Configurations".into(),
                    suggestion: "Manage shell configurations through https://github.com/juspay/nixos-unified-template".into(),
                }
            },
            required: false,
        };
        vec![check]
    }
}

fn check_shell_configuration() -> bool {
    // Define known dotfiles for different shells
    let mut shell_dotfiles = HashMap::new();
    shell_dotfiles.insert("zsh", vec![".zshrc"]);
    shell_dotfiles.insert("bash", vec![".bashrc", ".bash_profile", ".profile"]);
    //
    are_shell_dotfiles_nix_managed("zsh", &shell_dotfiles)
        || are_shell_dotfiles_nix_managed("bashrc", &shell_dotfiles)
}

// check if dotfile(s) for a given shell points to /nix/store
fn are_shell_dotfiles_nix_managed(shell: &str, dotfiles: &HashMap<&str, Vec<&str>>) -> bool {
    let home = env::var("HOME").unwrap_or_default();
    if let Some(shell_files) = dotfiles.get(shell) {
        shell_files.iter().all(|dotfile| {
            let path = PathBuf::from(&home).join(dotfile);
            if let Ok(canonical_path) = std::fs::canonicalize(&path) {
                canonical_path.starts_with("/nix/store/")
            } else {
                false
            }
        })
    } else {
        false
    }
}
