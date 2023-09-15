use nix_rs::{
    env::{self, AppleEmulation, MacOSArch, NixSystem},
    info,
};
use serde::{Deserialize, Serialize};

use crate::traits::{Check, CheckResult, Checkable};

/// Check if Nix is being run under rosetta emulation
///
/// Enabled only on ARM macs.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Rosetta {
    enable: bool,
}

impl Default for Rosetta {
    fn default() -> Self {
        Self { enable: true }
    }
}

impl Checkable for Rosetta {
    fn check(&self, _nix_info: &info::NixInfo, nix_env: &env::NixEnv) -> Option<Check> {
        if !self.enable {
            return None;
        }
        let emulation = get_apple_emulation(&nix_env.nix_system)?;
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
        };
        Some(check)
    }
}

/// Return [AppleEmulation]. Return None if not an ARM mac.
fn get_apple_emulation(system: &NixSystem) -> Option<AppleEmulation> {
    match system {
        NixSystem::MacOS {
            nix_darwin: _,
            arch: MacOSArch::Arm64(apple_emulation),
        } => Some(apple_emulation.clone()),
        _ => None,
    }
}
