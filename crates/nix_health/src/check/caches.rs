#[cfg(feature = "ssr")]
use nix_rs::{env, info};
use serde::{Deserialize, Serialize};
use url::Url;

#[cfg(feature = "ssr")]
use crate::traits::*;

/// Check that [nix_rs::config::NixConfig::substituters] is set to a good value.
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
#[serde(rename_all = "kebab-case")]
#[serde(default)]
pub struct Caches {
    pub required: Vec<Url>,
}

impl Default for Caches {
    fn default() -> Self {
        Caches {
            required: vec![Url::parse("https://cache.nixos.org").unwrap()],
        }
    }
}

#[cfg(feature = "ssr")]
impl Checkable for Caches {
    fn check(&self, nix_info: &info::NixInfo, nix_env: &env::NixEnv) -> Vec<Check> {
        let val = &nix_info.nix_config.substituters.value;
        let missing_caches = self
            .required
            .iter()
            .filter(|required_cache| !val.contains(required_cache))
            .collect::<Vec<_>>();
        let result = if missing_caches.is_empty() {
            CheckResult::Green
        } else {
            CheckResult::Red {
                msg: format!(
                    "You are missing some required caches: {}",
                    missing_caches
                        .iter()
                        .map(|url| url.to_string())
                        .collect::<Vec<_>>()
                        .join(" ")
                ),
                suggestion: format!(
                    "Caches can be added in your {}. Cachix caches can also be added using `cachix use <name>`.",
                    nix_env.os.configuration_nix_label().unwrap_or("/etc/nix/nix.conf".to_string())
                )
            }
        };
        let check = Check {
            title: "Nix Caches in use".to_string(),
            info: format!(
                "substituters = {}",
                val.iter()
                    .map(|url| url.to_string())
                    .collect::<Vec<_>>()
                    .join(" ")
            ),
            result,
            required: true,
        };
        vec![check]
    }
}
