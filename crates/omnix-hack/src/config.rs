use serde::Deserialize;

use nix_rs::{command::NixCmd, flake::url::FlakeUrl};
use omnix_common::config::OmConfig;

use crate::readme::Readme;

#[derive(Debug, Deserialize, Clone)]
pub struct HackConfig {
    pub cache: CacheConfig,
    pub readme: Readme,
}

#[derive(Debug, Deserialize, Clone)]
pub struct CacheConfig {
    pub cachix: CachixConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct CachixConfig {
    /// If enabled, configure environment to use the cache.
    pub enable: bool,
    /// Name of the cachix cache (https://<name>.cachix.org)
    pub name: String,
    /// The read-only auth token to use if this is a private cache
    ///
    /// If provided, will run `cachix authtoken <auth_token>`.
    pub auth_token: Option<String>,
}

impl HackConfig {
    pub async fn from_flake(url: &FlakeUrl) -> anyhow::Result<Self> {
        let v = OmConfig::<Self>::from_flake_url(NixCmd::get().await, url, &["om.hack"])
            .await?
            .config;
        v.get("default")
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("Missing key default for om.hack"))
    }
}
