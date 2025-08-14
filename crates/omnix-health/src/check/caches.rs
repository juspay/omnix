use nix_rs::info;
use serde::{Deserialize, Serialize};
use url::Url;

use crate::traits::*;

/// Specification for different types of Nix binary caches.
///
/// **Important:** `om health` only checks if these caches are configured, it does NOT install them.
/// Use `om develop` for automatic installation of Cachix and Attic caches.
#[derive(Debug, Serialize, PartialEq, Eq, Clone)]
#[serde(rename_all = "kebab-case")]
pub enum CacheSpec {
    /// Regular HTTP/HTTPS cache URL (must be added manually to Nix config)
    Regular(String),
    /// Cachix cache name (automatically installed by `om develop` via `cachix use`)
    Cachix(String),
    /// Attic cache (automatically installed by `om develop` via `attic login` + `attic use`)
    Attic {
        server_name: String,
        cache_url: String,
    },
}

impl<'de> serde::Deserialize<'de> for CacheSpec {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Ok(CacheSpec::from_url_string(&s))
    }
}

impl CacheSpec {
    pub fn from_url_string(url_str: &str) -> Self {
        if let Some(attic_cache) = AtticCache::from_url_string(url_str) {
            CacheSpec::Attic {
                server_name: attic_cache.server_name,
                cache_url: attic_cache.cache_url,
            }
        } else if let Some(cachix_cache) = CachixCache::from_url_string(url_str) {
            CacheSpec::Cachix(cachix_cache.0)
        } else {
            CacheSpec::Regular(url_str.to_string())
        }
    }

    pub fn to_url_string(&self) -> String {
        match self {
            CacheSpec::Regular(url) => url.clone(),
            CacheSpec::Cachix(name) => format!("https://{}.cachix.org", name),
            CacheSpec::Attic {
                server_name,
                cache_url,
            } => {
                format!("attic+{}+{}", server_name, cache_url)
            }
        }
    }
}

/// Check that [nix_rs::config::NixConfig::substituters] is set to a good value.
///
/// **Note:** This module only *checks* for missing caches. For automatic cache installation:
/// - Cachix caches: Use `om develop` which will run `cachix use <name>`
/// - Attic caches: Use `om develop` which will run `attic login` and `attic use`
/// - Regular URLs: Must be added manually to your Nix configuration
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
#[serde(rename_all = "kebab-case")]
#[serde(default)]
pub struct Caches {
    pub required: Vec<CacheSpec>,
}

impl Default for Caches {
    fn default() -> Self {
        Caches {
            required: vec![CacheSpec::Regular("https://cache.nixos.org".to_string())],
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
            let missing_cache_strings: Vec<String> = missing_caches
                .iter()
                .map(|cache| cache.to_url_string())
                .collect();

            CheckResult::Red {
                msg: format!(
                    "You are missing some required caches: {}",
                    missing_cache_strings.join(" ")
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
    pub fn get_missing_caches(&self, nix_info: &info::NixInfo) -> Vec<CacheSpec> {
        let val = &nix_info.nix_config.substituters.value;
        self.required
            .iter()
            .filter(|cache_spec| {
                match cache_spec {
                    CacheSpec::Regular(url_str) => {
                        Url::parse(url_str)
                            .map(|url| !val.contains(&url))
                            .unwrap_or(true) // If we can't parse it, consider it missing
                    }
                    CacheSpec::Cachix(name) => {
                        let cachix_url = format!("https://{}.cachix.org", name);
                        Url::parse(&cachix_url)
                            .map(|url| !val.contains(&url))
                            .unwrap_or(true)
                    }
                    CacheSpec::Attic { cache_url, .. } => {
                        Url::parse(cache_url)
                            .map(|url| !val.contains(&url))
                            .unwrap_or(true) // If we can't parse it, consider it missing
                    }
                }
            })
            .cloned()
            .collect()
    }
}

/// Cachix cache configuration for binary cache management.
///
/// **Usage:** These methods are called automatically by `om develop` when cachix caches
/// are detected in the health check. `om health` only validates cache availability.
pub struct CachixCache(pub String);

impl CachixCache {
    /// Parse the https URL into a CachixCache
    pub fn from_url_string(url_str: &str) -> Option<Self> {
        // Parse https://foo.cachix.org into CachixCache("foo")
        // If domain is not cachix.org, return None.
        let url = Url::parse(url_str).ok()?;
        let host = url.host_str()?;

        if host.ends_with(".cachix.org") {
            let name = host.split('.').next()?.to_string();
            Some(CachixCache(name))
        } else {
            None
        }
    }

    /// Run `cachix use` for this cache.
    ///
    /// **Called by:** `om develop` only (not `om health`)
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

/// Attic cache configuration for binary cache management.
///
/// **Usage:** These methods are called automatically by `om develop` when attic caches
/// are detected in the health check. `om health` only validates cache availability.
pub struct AtticCache {
    pub server_name: String,
    pub cache_url: String,
    pub cache_name: String,
}

impl AtticCache {
    /// Parse the attic+servername+url into an AtticCache.
    ///
    /// Format: `attic+<server_name>+<cache_url>`
    /// Example: `attic+chutney+https://cache.nixos.asia/oss`
    ///
    /// **Note:** Handles URLs with '+' characters correctly by finding URL schemes.
    pub fn from_url_string(url_str: &str) -> Option<Self> {
        // Parse attic+chutney+https://cache.nixos.asia/oss into AtticCache
        // We need robust parsing that handles '+' characters in URLs
        let after_prefix = url_str.strip_prefix("attic+")?;

        // Find where the URL starts by looking for URL schemes
        let url_start = after_prefix
            .find("https://")
            .or_else(|| after_prefix.find("http://"))?;

        // Everything before the URL is the server name, minus the trailing '+'
        let server_name = after_prefix[..url_start].strip_suffix("+")?;

        // Reject empty server names
        if server_name.is_empty() {
            return None;
        }
        let cache_url = &after_prefix[url_start..];

        let parsed_url = Url::parse(cache_url).ok()?;
        let cache_name = parsed_url
            .path_segments()?
            .last()
            .filter(|name| !name.is_empty())?;

        Some(AtticCache {
            server_name: server_name.to_string(),
            cache_url: cache_url.to_string(),
            cache_name: cache_name.to_string(),
        })
    }

    /// Run `attic login` for this cache server.
    ///
    /// **Called by:** `om develop` only (not `om health`)
    /// **Requires:** ATTIC_LOGIN_TOKEN environment variable
    pub async fn attic_login(&self) -> anyhow::Result<()> {
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

    /// Run `attic use` for this cache (assumes login has already been done).
    ///
    /// **Called by:** `om develop` only (not `om health`)
    /// **Prerequisite:** Must call `attic_login()` first
    pub async fn attic_use(&self) -> anyhow::Result<()> {
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

    /// Get the attic binary path, preferring the built-in one if available
    fn get_attic_bin() -> String {
        // Try to use the built-in attic binary first (available when built with Nix)
        std::env::var("ATTIC_BIN").unwrap_or_else(|_| "attic".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_spec_parsing() {
        // Test valid attic URL with new syntax
        let url_str = "attic+chutney+https://cache.nixos.asia/oss";
        let cache_spec = CacheSpec::from_url_string(url_str);
        assert_eq!(
            cache_spec,
            CacheSpec::Attic {
                server_name: "chutney".to_string(),
                cache_url: "https://cache.nixos.asia/oss".to_string(),
            }
        );
        assert_eq!(cache_spec.to_url_string(), url_str);

        // Test cachix URL
        let url_str = "https://foo.cachix.org";
        let cache_spec = CacheSpec::from_url_string(url_str);
        assert_eq!(cache_spec, CacheSpec::Cachix("foo".to_string()));
        assert_eq!(cache_spec.to_url_string(), url_str);

        // Test regular URL
        let url_str = "https://cache.nixos.org";
        let cache_spec = CacheSpec::from_url_string(url_str);
        assert_eq!(cache_spec, CacheSpec::Regular(url_str.to_string()));
        assert_eq!(cache_spec.to_url_string(), url_str);
    }

    #[test]
    fn test_attic_cache_parsing() {
        // Test valid attic URL with new syntax
        let url_str = "attic+chutney+https://cache.nixos.asia/oss";
        let attic_cache = AtticCache::from_url_string(url_str).unwrap();
        assert_eq!(attic_cache.server_name, "chutney");
        assert_eq!(attic_cache.cache_url, "https://cache.nixos.asia/oss");
        assert_eq!(attic_cache.cache_name, "oss");

        // Test URL with '+' characters in query parameters
        let url_str = "attic+myserver+https://example.com/cache?search=rust+programming+guide";
        let attic_cache = AtticCache::from_url_string(url_str).unwrap();
        assert_eq!(attic_cache.server_name, "myserver");
        assert_eq!(
            attic_cache.cache_url,
            "https://example.com/cache?search=rust+programming+guide"
        );
        assert_eq!(attic_cache.cache_name, "cache");

        // Test URL with '+' characters in path
        let url_str = "attic+production+https://cdn.example.com/cache+v2/packages";
        let attic_cache = AtticCache::from_url_string(url_str).unwrap();
        assert_eq!(attic_cache.server_name, "production");
        assert_eq!(
            attic_cache.cache_url,
            "https://cdn.example.com/cache+v2/packages"
        );
        assert_eq!(attic_cache.cache_name, "packages");

        // Test HTTP (not HTTPS)
        let url_str = "attic+local+http://localhost:8080/cache";
        let attic_cache = AtticCache::from_url_string(url_str).unwrap();
        assert_eq!(attic_cache.server_name, "local");
        assert_eq!(attic_cache.cache_url, "http://localhost:8080/cache");
        assert_eq!(attic_cache.cache_name, "cache");

        // Test invalid URL (not attic)
        let url_str = "https://foo.cachix.org";
        assert!(AtticCache::from_url_string(url_str).is_none());

        // Test cachix URL parsing still works
        let cachix_cache = CachixCache::from_url_string(url_str).unwrap();
        assert_eq!(cachix_cache.0, "foo");

        // Test invalid attic URL (missing parts)
        let url_str = "attic+onlyonepart";
        assert!(AtticCache::from_url_string(url_str).is_none());

        // Test invalid attic URL (no URL scheme)
        let url_str = "attic+server+invalid-url";
        assert!(AtticCache::from_url_string(url_str).is_none());

        // Test invalid attic URL (missing server name)
        let url_str = "attic++https://example.com/cache";
        assert!(AtticCache::from_url_string(url_str).is_none());
    }
}
