use nix_rs::info;
use serde::{Deserialize, Serialize};
use std::{env, path::PathBuf};

use crate::traits::{Check, CheckResult, Checkable};

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct HomeManager {}

impl Checkable for HomeManager {
    fn check(&self, _: &info::NixInfo, _: Option<&nix_rs::flake::url::FlakeUrl>) -> Vec<Check> {
        let is_nix_store = get_shell_configuration_path(".zshrc").starts_with("/nix/store/")
            || get_shell_configuration_path(".bashrc").starts_with("/nix/store/");

        let check = Check {
            title: "Home Manager Active".to_string(),
            info: "Shell Configurations are managed by Nix".to_string(),
            result: if is_nix_store {
                CheckResult::Green
            } else {
                CheckResult::Red {
                    msg: "Home Manager".into(),
                    suggestion: "Install and activate home-manager through https://github.com/juspay/nixos-unified-template".into(),
                }
            },
            required: false,
        };
        vec![check]
    }
}

fn get_shell_configuration_path(shell: &str) -> PathBuf {
    let home = env::var("HOME").unwrap_or_default();
    let path = PathBuf::from(home).join(shell);
    let canonical_path = std::fs::canonicalize(path);
    canonical_path.unwrap_or_default()
}
