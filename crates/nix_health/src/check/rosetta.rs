use nix_rs::{env, info};
use serde::{Deserialize, Serialize};

use crate::traits::{Check, CheckResult, Checkable};

/// Check if Nix is being run under rosetta emulation on macOS
#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct Rosetta {}

impl Checkable for Rosetta {
    fn check(&self, _nix_info: &info::NixInfo, nix_env: &env::NixEnv) -> Option<Check> {
        let rosetta = match nix_env.nix_system {
            env::NixSystem::MacOS {
                nix_darwin: _,
                rosetta,
            } => rosetta,
            _ => false,
        };
        let check = Check {
            title: "Rosetta Disabled".to_string(),
            info: format!("rosetta enabled = {}", rosetta),
            result: if rosetta {
                CheckResult::Red {
                    msg: "Rosetta emulation can slow down builds".to_string(),
                    suggestion: "Remove rosetta, see the comment by @hruan here: https://developer.apple.com/forums/thread/669486".to_string(),
                }
            } else {
                CheckResult::Green
            },
        };
        Some(check)
    }
}
