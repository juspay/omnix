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
pub struct Caches {
    pub substituers: ConfigVal<Vec<Url>>,
    pub config: CachesConfig
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CachesConfig {
    pub required_caches: Vec<Url>
}

impl Default for CachesConfig {
    fn default() -> Self {
        CachesConfig {
            required_caches: vec![
                Url::parse("https://cache.nixos.org").unwrap(),
                // TODO: Hardcoding this to test failed reports
                Url::parse("https://nix-community.cachix.org").unwrap(),
            ]
        }
    }
}


impl Check for Caches {
    fn check(nix_info: &info::NixInfo, _nix_env: &env::NixEnv) -> Self {
        Caches{
            substituers: nix_info.nix_config.substituters.clone(),
            // TODO: Get user input
            config: CachesConfig::default()
        }
    }
    fn name(&self) -> &'static str {
        "Nix Caches in use"
    }
    fn report(&self) -> Report<WithDetails> {
        let val = &self.substituers.value;
        for required_cache in &self.config.required_caches {
            if !val.contains(required_cache) {
                return Report::Red(WithDetails {
                    msg: format!("You are missing a required cache: {}", required_cache),
                    // TODO: Suggestion should be smart. Use 'cachix use' if a cachix cache.
                    suggestion: "Add substituters in /etc/nix/nix.conf or use 'cachix use'".into(),
                });
            }
        }
        Report::Green
    }
}

impl Display for Caches {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "substituters = {}",
            self.substituers
                .value
                .iter()
                .map(|url| url.to_string())
                .collect::<Vec<String>>()
                .join(" ")
        )
    }
}
