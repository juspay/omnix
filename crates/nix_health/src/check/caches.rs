use nix_rs::{env, info};
use serde::{Deserialize, Serialize};
use url::Url;

use crate::traits::*;

/// Check that [nix_rs::config::NixConfig::substituters] is set to a good value.
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct Caches {
    #[serde(default = "default_caches")]
    pub required_caches: Vec<Url>,
}

impl Default for Caches {
    fn default() -> Self {
        Caches {
            required_caches: default_caches(),
        }
    }
}

fn default_caches() -> Vec<Url> {
    vec![
        Url::parse("https://cache.nixos.org").unwrap(),
        // TODO: Hardcoding this for now, so as to test failed reports
        Url::parse("https://nix-community.cachix.org").unwrap(),
    ]
}

impl Checkable for Caches {
    fn check(&self, nix_info: &info::NixInfo, _nix_env: &env::NixEnv) -> Option<Check> {
        let val = &nix_info.nix_config.substituters.value;
        let result = if self.required_caches.iter().all(|c| val.contains(c)) {
            CheckResult::Green
        } else {
            CheckResult::Red {
                msg: format!(
                    "You are missing a required cache: {}",
                    self.required_caches
                        .iter()
                        .find(|required_cache| !val.contains(required_cache))
                        .unwrap()
                ),
                suggestion: "Add in /etc/nix/nix.conf or use 'cachix use'".to_string(),
            }
        };
        let check = Check {
            title: "Nix Caches in use".to_string(),
            info: format!(
                "substituters = {}",
                val.iter()
                    .map(|url| url.to_string())
                    .collect::<Vec<String>>()
                    .join(" ")
            ),
            result,
        };
        Some(check)
    }
}
