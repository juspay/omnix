use serde::{Deserialize, Serialize};

use nix_rs::{env::NixInstallerVersion, info};

use crate::traits::*;

/// Check that <https://github.com/DeterminateSystems/nix-installer> is used to install Nix on non-NixOS systems
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct NixInstaller {
    pub enable: bool,
    pub required: bool,
}

impl Default for NixInstaller {
    fn default() -> Self {
        NixInstaller {
            enable: true,
            required: true,
        }
    }
}

impl Checkable for NixInstaller {
    fn check(
        &self,
        nix_info: &info::NixInfo,
        _: Option<&nix_rs::flake::url::FlakeUrl>,
    ) -> Vec<Check> {
        let mut checks = vec![];
        if let (true, installer) = (self.enable, &nix_info.nix_env.installer) {
            // Skip if on NixOS
            if let nix_rs::env::OS::NixOS = &nix_info.nix_env.os {
                return checks;
            }
            let check = Check {
                title: "Nix installer used".to_string(),
                info: format!("{} was used to install Nix.", installer),
                result: match installer {
                    nix_rs::env::NixInstaller::Other => CheckResult::Red {
                        msg: "You are not using the recommended way to install Nix.".to_string(),
                        suggestion: "Uninstall the existing installation: https://nix.dev/manual/nix/2.18/installation/uninstall.html#macos. Then follow https://nixos.asia/en/install".into(),
                    },
                    nix_rs::env::NixInstaller::DetSys { version } => if version < (&NixInstallerVersion {major: 0, minor: 14, patch: 0}) {
                        CheckResult::Red {
                            msg: format!("Your nix-installer version ({}) is too old; we require at least 0.14", version),
                            suggestion: "Uninstall from https://github.com/DeterminateSystems/nix-installer?tab=readme-ov-file#uninstalling and reinstall".into(),
                        }
                    } else {
                        CheckResult::Green
                    },
                },
                required: self.required,
            };
            checks.push(check);
        }
        checks
    }
}
