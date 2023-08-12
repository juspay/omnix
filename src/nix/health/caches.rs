use serde::{Deserialize, Serialize};

use crate::nix::info;

use super::{Check, Report};

// [NixConfig::max_job]]
#[derive(Debug, PartialEq, PartialOrd, Eq, Ord, Serialize, Deserialize, Clone)]
pub struct Caches(Vec<String>);

impl Check for Caches {
    fn check(info: &info::NixInfo) -> Self {
        Caches(info.nix_config.substituters.value.clone())
    }
    fn name(&self) -> &'static str {
        "Nix Caches in use"
    }
    fn report(&self) -> Report {
        // TODO: This should use lenient URL match
        if self.0.contains(&"https://cache.nixos.org/".to_string()) {
            Report::Green
        } else {
            Report::Red {
                msg: "You are missing the official cache",
                suggestion: "Try doing something",
            }
        }
    }
}
