use nix_rs::{env::OS, info};
use serde::{Deserialize, Serialize};

use crate::traits::{Check, CheckResult, Checkable};

/// Check if Nix is being run under rosetta emulation
///
/// Enabled only on ARM macs.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct Rosetta {
    enable: bool,
    required: bool,
}

impl Default for Rosetta {
    fn default() -> Self {
        Self {
            enable: true,
            required: true,
        }
    }
}

impl Checkable for Rosetta {
    fn check(
        &self,
        nix_info: &info::NixInfo,
        _: Option<&nix_rs::flake::url::FlakeUrl>,
    ) -> Vec<(&'static str, Check)> {
        let mut checks = vec![];
        if let (true, Some(emulation)) = (self.enable, get_apple_emulation(&nix_info.nix_env.os)) {
            let check = Check {
                title: "Rosetta Not Active".to_string(),
                info: format!("apple emulation = {:?}", emulation),
                result: if emulation {
                    CheckResult::Red {
                    msg: "Rosetta emulation will slow down Nix builds".to_string(),
                    // NOTE: This check assumes that `omnix` was installed via `nix`, thus assuming `nix` is also translated using Rosetta.
                    // Hence, the suggestion to re-install nix.
                    suggestion: "Disable Rosetta for your terminal (Right-click on your terminal icon in `Finder`, choose `Get Info` and un-check `Open using Rosetta`). Uninstall nix: <https://nixos.asia/en/gotchas/macos-upgrade>. And re-install for `aarch64-darwin`: <https://nixos.asia/en/install>".to_string(),
                }
                } else {
                    CheckResult::Green
                },
                required: self.required,
            };
            checks.push(("rosetta", check));
        };
        checks
    }
}

/// Return [true] if the current binary is translated using Rosetta. Return None if not an ARM mac.
fn get_apple_emulation(system: &OS) -> Option<bool> {
    match system {
        OS::MacOS {
            nix_darwin: _,
            arch: _,
            proc_translated: is_proc_translated,
        } => Some(is_proc_translated.clone()),
        _ => None,
    }
}
