use std::fmt::Display;

use nix_rs::{config::ConfigVal, env, info};
use serde::{Deserialize, Serialize};
use url::Url;

use crate::{
    report::{Report, WithDetails},
    traits::Check,
};

/// Check that [nix_rs::config::NixConfig::substituters] is set to a good value.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Caches(pub ConfigVal<Vec<Url>>);

impl Check for Caches {
    fn check(nix_info: &info::NixInfo, _nix_env: &env::NixEnv) -> Self {
        Caches(nix_info.nix_config.substituters.clone())
    }
    fn name(&self) -> &'static str {
        "Nix Caches in use"
    }
    fn report(&self) -> Report<WithDetails> {
        let val = &self.0.value;
        // TODO: Hardcoding this to test failed reports
        // TODO: Make this customizable in a flake
        let required_cache = Url::parse("https://nix-community.cachix.org").unwrap();
        if val.contains(&Url::parse("https://cache.nixos.org").unwrap()) {
            if val.contains(&required_cache) {
                Report::Green
            } else {
                Report::Red(WithDetails {
                    msg: format!("You are missing a required cache: {}", required_cache),
                    // TODO: Suggestion should be smart. Use 'cachix use' if a cachix cache.
                    suggestion: "Add substituters in /etc/nix/nix.conf or use 'cachix use'".into(),
                })
            }
        } else {
            Report::Red(WithDetails {
                msg: "You are missing the official cache".into(),
                suggestion: "Try looking in /etc/nix/nix.conf".into(),
            })
        }
    }
}

impl Display for Caches {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "substituters = {}",
            self.0
                .value
                .iter()
                .map(|url| url.to_string())
                .collect::<Vec<String>>()
                .join(" ")
        )
    }
}
