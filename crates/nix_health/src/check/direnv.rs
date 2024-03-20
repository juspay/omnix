use nix_rs::{flake::url::FlakeUrl, info};
use semver::VersionReq;
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
    // TODO: This code flow is confusing; refactor for legibility.
    fn check(&self, _nix_info: &info::NixInfo, flake_url: Option<FlakeUrl>) -> Vec<Check> {
        let mut checks = vec![];
        if !self.enable {
            return checks;
        }

        let direnv_install_result = direnv::DirenvInstall::detect();

        let direnv_install_check = install_check(&direnv_install_result, self.required);
        let direnv_installed = direnv_install_check.result.green();
        checks.push(direnv_install_check);

        if !direnv_installed {
            return checks;
        }

        // FIXME: Avoid unwrap, by refactoring code flow.
        let direnv_install = direnv_install_result.as_ref().unwrap();

        let direnv_version = version_check(direnv_install);
        let direnv_version_green = direnv_version.result.green();
        checks.push(direnv_version);

        // If direnv is installed, check for version and then allowed_check
        if direnv_version_green {
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

        checks
    }
}

/// [Check] that direnv was installed.
fn install_check(direnv_install: &anyhow::Result<direnv::DirenvInstall>, required: bool) -> Check {
    Check {
        title: "Direnv installation".to_string(),
        info: format!(
            "direnv location = {:?}",
            direnv_install.as_ref().ok().map(|s| &s.bin_path)
        ),
        result: match direnv_install {
            Ok(_direnv_status) => CheckResult::Green,
            Err(e) => CheckResult::Red {
                msg: format!("Unable to locate direnv ({})", e),
                suggestion: "Install direnv <https://nixos.asia/en/direnv#setup>".to_string(),
            },
        },
        required,
    }
}

/// [Check] that direnv version >= 2.33.0 for `direnv status --json` support
fn version_check(direnv_install: &direnv::DirenvInstall) -> Check {
    let req = VersionReq::parse(">=2.33.0").unwrap();
    let suggestion = format!("Upgrade direnv to {}", req);
    let direnv_version = &direnv_install.version;
    Check {
        title: "Direnv version".to_string(),
        info: format!("direnv version = {:?}", direnv_version),
        // Use semver to compare versions
        result: if req.matches(direnv_version) {
            CheckResult::Green
        } else {
            CheckResult::Red {
                msg: format!("direnv version {} is not supported", direnv_version),
                suggestion,
            }
        },
        required: false,
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
