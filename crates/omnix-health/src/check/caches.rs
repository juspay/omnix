use nix_rs::info;
use serde::{Deserialize, Serialize};
use url::Url;

use crate::traits::*;

/// Check that [nix_rs::config::NixConfig::substituters] is set to a good value.
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
#[serde(rename_all = "kebab-case")]
#[serde(default)]
pub struct Caches {
    pub required: Vec<String>,
}

impl Default for Caches {
    fn default() -> Self {
        Caches {
            required: vec!["https://cache.nixos.org".to_string()],
        }
    }
}

impl Checkable for Caches {
    fn check(
        &self,
        nix_info: &info::NixInfo,
        _: Option<&nix_rs::flake::url::FlakeUrl>,
    ) -> Vec<(&'static str, Check)> {
        let missing_caches = self.get_missing_caches(nix_info);
        let result = if missing_caches.is_empty() {
            CheckResult::Green
        } else {
            CheckResult::Red {
                msg: format!(
                    "You are missing some required caches: {}",
                    missing_caches.join(" ")
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

        vec![("caches", check)]
    }
}

impl Caches {
    /// Get subset of required caches not already in use
    pub fn get_missing_caches(&self, nix_info: &info::NixInfo) -> Vec<String> {
        let val = &nix_info.nix_config.substituters.value;
        self.required
            .iter()
            .filter(|required_cache_str| {
                // For attic caches, we need to extract the actual URL to check against substituters
                if let Some(attic_cache) = AtticCache::from_url_string(required_cache_str) {
                    // Check if the actual cache URL is in substituters
                    if let Ok(url) = Url::parse(&attic_cache.cache_url) {
                        !val.contains(&url)
                    } else {
                        true // If we can't parse it, consider it missing
                    }
                } else {
                    // For regular URLs, parse and check
                    if let Ok(url) = Url::parse(required_cache_str) {
                        !val.contains(&url)
                    } else {
                        true // If we can't parse it, consider it missing
                    }
                }
            })
            .cloned()
            .collect()
    }
}

pub struct CachixCache(pub String);

impl CachixCache {
    /// Parse the https URL into a CachixCache
    pub fn from_url_string(url_str: &str) -> Option<Self> {
        // Parse https://foo.cachix.org into CachixCache("foo")
        // If domain is not cachix.org, return None.
        if let Ok(url) = Url::parse(url_str) {
            let host = url.host_str()?;
            if host.ends_with(".cachix.org") {
                Some(CachixCache(host.split('.').next()?.to_string()))
            } else {
                None
            }
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

pub struct AtticCache {
    pub server_name: String,
    pub cache_url: String,
    pub cache_name: String,
}

impl AtticCache {
    /// Parse the attic+servername+url into an AtticCache
    pub fn from_url_string(url_str: &str) -> Option<Self> {
        // Parse attic+chutney+https://cache.nixos.asia/oss into AtticCache
        if url_str.starts_with("attic+") {
            // Split by '+' to get ["attic", "servername", "https://..."]
            let parts: Vec<&str> = url_str.split('+').collect();
            if parts.len() >= 3 && parts[0] == "attic" {
                let server_name = parts[1].to_string();
                // Rejoin the remaining parts to reconstruct the URL
                let cache_url = parts[2..].join("+");

                // Extract cache name from the path (everything after the last '/')
                if let Ok(parsed_url) = Url::parse(&cache_url) {
                    if let Some(path) = parsed_url.path_segments() {
                        if let Some(cache_name) = path.last() {
                            if !cache_name.is_empty() {
                                return Some(AtticCache {
                                    server_name,
                                    cache_url,
                                    cache_name: cache_name.to_string(),
                                });
                            }
                        }
                    }
                }
            }
        }
        None
    }

    /// Run `attic login` and then `attic use` for this cache
    pub async fn attic_use(&self) -> anyhow::Result<()> {
        // First, login to the attic server
        self.attic_login().await?;

        // Then use the cache with servername:cachename format
        let cache_spec = format!("{}:{}", self.server_name, self.cache_name);
        let attic_bin = Self::get_attic_bin();
        let mut cmd = tokio::process::Command::new(&attic_bin);
        cmd.arg("use").arg(&cache_spec);
        let status = cmd.spawn()?.wait().await?;
        if !status.success() {
            anyhow::bail!("Failed to run `attic use {}`", cache_spec);
        }
        Ok(())
    }

    /// Run `attic login` for this cache server
    async fn attic_login(&self) -> anyhow::Result<()> {
        let token = std::env::var("ATTIC_LOGIN_TOKEN").unwrap_or_default();
        let attic_bin = Self::get_attic_bin();

        let mut cmd = tokio::process::Command::new(&attic_bin);
        cmd.arg("login")
            .arg(&self.server_name)
            .arg(&self.cache_url)
            .arg(&token);

        let status = cmd.spawn()?.wait().await?;
        if !status.success() {
            anyhow::bail!(
                "Failed to run `attic login {} {}`",
                self.server_name,
                self.cache_url
            );
        }
        Ok(())
    }

    /// Get the attic binary path, preferring the built-in one if available
    fn get_attic_bin() -> String {
        // Try to use the built-in attic binary first (available when built with Nix)
        if let Ok(attic_bin) = std::env::var("ATTIC_BIN") {
            attic_bin
        } else {
            // Fallback to system attic
            "attic".to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_attic_cache_parsing() {
        // Test valid attic URL with new syntax
        let url_str = "attic+chutney+https://cache.nixos.asia/oss";
        let attic_cache = AtticCache::from_url_string(url_str).unwrap();
        assert_eq!(attic_cache.server_name, "chutney");
        assert_eq!(attic_cache.cache_url, "https://cache.nixos.asia/oss");
        assert_eq!(attic_cache.cache_name, "oss");

        // Test invalid URL (not attic)
        let url_str = "https://foo.cachix.org";
        assert!(AtticCache::from_url_string(url_str).is_none());

        // Test cachix URL parsing still works
        let cachix_cache = CachixCache::from_url_string(url_str).unwrap();
        assert_eq!(cachix_cache.0, "foo");

        // Test invalid attic URL (missing parts)
        let url_str = "attic+onlyonepart";
        assert!(AtticCache::from_url_string(url_str).is_none());
    }
}
