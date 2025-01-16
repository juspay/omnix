use std::collections::HashMap;

use nix_rs::info;
use serde::{Deserialize, Serialize};
use url::Url;

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

impl Checkable for Caches {
    fn check(
        &self,
        nix_info: &info::NixInfo,
        _: Option<&nix_rs::flake::url::FlakeUrl>,
    ) -> HashMap<String, Check> {
        let missing_caches = self.get_missing_caches(nix_info);
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
                    "Caches can be added in your {} (see https://nixos.wiki/wiki/Binary_Cache#Using_a_binary_cache). Cachix caches can also be added using `nix run nixpkgs#cachix use <name>`.",
                    nix_info.nix_env.os.nix_config_label()
                )
            }
        };
        let check = Check {
            title: "Nix Caches in use".to_string(),
            info: format!(
                "substituters = {}",
                nix_info
                    .nix_config
                    .substituters
                    .value
                    .iter()
                    .map(|url| url.to_string())
                    .collect::<Vec<_>>()
                    .join(" ")
            ),
            result,
            required: true,
        };

        let mut checks_map = HashMap::new();
        checks_map.insert("caches".to_string(), check);
        checks_map
    }
}

impl Caches {
    /// Get subset of required caches not already in use
    pub fn get_missing_caches(&self, nix_info: &info::NixInfo) -> Vec<Url> {
        let val = &nix_info.nix_config.substituters.value;
        self.required
            .iter()
            .filter(|required_cache| !val.contains(required_cache))
            .cloned()
            .collect()
    }
}

pub struct CachixCache(pub String);

impl CachixCache {
    /// Parse the https URL into a CachixCache
    pub fn from_url(url: &Url) -> Option<Self> {
        // Parse https://foo.cachix.org into CachixCache("foo")
        // If domain is not cachix.org, return None.
        let host = url.host_str()?;
        if host.ends_with(".cachix.org") {
            Some(CachixCache(host.split('.').next()?.to_string()))
        } else {
            None
        }
    }

    /// Run `cachix use` for this cache
    pub async fn cachix_use(&self) -> anyhow::Result<()> {
        let mut cmd = tokio::process::Command::new(env!("CACHIX_BIN"));
        cmd.arg("use").arg(&self.0);
        let status = cmd.spawn()?.wait().await?;
        if !status.success() {
            anyhow::bail!("Failed to run `cachix use {}`", self.0);
        }
        Ok(())
    }
}
