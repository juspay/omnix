use nix_rs::{flake::url::FlakeUrl, info};
use serde::{Deserialize, Serialize};

use crate::traits::{Check, CheckResult, Checkable};

/// Check if direnv is installed
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default, rename_all = "kebab-case")]
pub struct Direnv {
    pub(crate) enable: bool,
    /// Whether to produce [Check::required] checks
    pub(crate) required: bool,
}

impl Default for Direnv {
    fn default() -> Self {
        Self {
            enable: true,
            required: false,
        }
    }
}

impl Checkable for Direnv {
    fn check(&self, _nix_info: &info::NixInfo, flake_url: Option<FlakeUrl>) -> Vec<Check> {
        let mut checks = vec![];
        if !self.enable {
            return checks;
        }

        let direnv_install_result = direnv::DirenvInstall::detect();
        checks.push(install_check(&direnv_install_result, self.required));

        match direnv_install_result.as_ref() {
            Err(_) => return checks,
            Ok(direnv_install) => {
                // If direnv is installed, check for version and then allowed_check
                // This check is currently only relevant if the flake is local and an `.envrc` exists.
                match flake_url.as_ref().and_then(|url| url.as_local_path()) {
                    None => {}
                    Some(local_path) => {
                        if local_path.join(".envrc").exists() {
                            checks.push(allowed_check(direnv_install, local_path, self.required));
                        }
                    }
                }
            }
        }

        checks
    }
}

/// [Check] that direnv was installed.
fn install_check(
    direnv_install_result: &Result<direnv::DirenvInstall, direnv::DirenvInstallError>,
    required: bool,
) -> Check {
    Check {
        title: "Direnv installation".to_string(),
        info: format!(
            "direnv location = {:?}",
            direnv_install_result.as_ref().ok().map(|s| &s.bin_path)
        ),
        result: match direnv_install_result {
            Ok(direnv_install) => is_path_in_nix_store_check_result(direnv_install),
            Err(e) => CheckResult::Red {
                msg: format!("Unable to locate direnv ({})", e),
                suggestion: "Install direnv <https://nixos.asia/en/direnv#setup>".to_string(),
            },
        },
        required,
    }
}

/// Verify that direnv binary is present under the Nix store.
fn is_path_in_nix_store_check_result(direnv_install: &direnv::DirenvInstall) -> CheckResult {
    let nix_store_path = std::path::Path::new("/nix/store");
    if direnv_install.canonical_path.starts_with(nix_store_path) {
        CheckResult::Green
    } else {
        CheckResult::Red {
            msg: "direnv is installed globally".to_string(),
            suggestion: "Install direnv via Nix, it will also manage shell integration. See <https://nixos.asia/en/direnv>"
                .to_string(),
        }
    }
}

/// [Check] that direnv was allowed on the local flake
fn allowed_check(
    direnv_install: &direnv::DirenvInstall,
    local_flake: &std::path::Path,
    required: bool,
) -> Check {
    let suggestion = format!("Run `direnv allow` under `{}`", local_flake.display());
    let direnv_allowed = direnv_install
        .status(local_flake)
        .map(|status| status.state.is_allowed());
    Check {
        title: "Direnv allowed".to_string(),
        info: format!("Local flake: {:?} (has .envrc and is allowed)", local_flake),
        result: match direnv_allowed {
            Ok(true) => CheckResult::Green,
            Ok(false) => CheckResult::Red {
                msg: "direnv was not allowed on this project".to_string(),
                suggestion,
            },
            Err(e) => CheckResult::Red {
                msg: format!("Unable to check direnv status: {}", e),
                suggestion,
            },
        },
        required,
    }
}
