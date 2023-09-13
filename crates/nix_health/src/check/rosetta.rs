use std::fmt::Display;

use nix_rs::{env, info};
use serde::{Deserialize, Serialize};

use crate::{
    report::{Report, WithDetails},
    traits::Check,
};

/// Check that [nix_rs::config::NixConfig::experimental_features] is set to a good value.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Rosetta(pub bool);

impl Check for Rosetta {
    fn check(_nix_info: &info::NixInfo, nix_env: &env::NixEnv) -> Self {
        let rosetta = match nix_env.nix_system {
            env::NixSystem::MacOS {
                nix_darwin: _,
                rosetta,
            } => rosetta,
            _ => false,
        };
        Rosetta(rosetta)
    }
    fn name(&self) -> &'static str {
        "Rosetta not in use"
    }
    fn report(&self) -> Report<WithDetails> {
        if self.0 {
            Report::Red(WithDetails {
                msg: "Rosetta emmulation enabled".to_string(),
                suggestion: "?".to_string(),
            })
        } else {
            Report::Green
        }
    }
}

impl Display for Rosetta {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "rosetta enabled = {}", self.0)
    }
}
