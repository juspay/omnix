use nix_rs::{
    env::{AppleEmulation, MacOSArch, OS},
    info,
};
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
                result: if emulation == AppleEmulation::Rosetta {
                    CheckResult::Red {
                    msg: "Rosetta emulation will slow down Nix builds".to_string(),
                    suggestion: "Remove rosetta, see the comment by @hruan here: https://developer.apple.com/forums/thread/669486".to_string(),
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
