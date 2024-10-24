use serde::Deserialize;

use nix_rs::{command::NixCmd, flake::url::FlakeUrl};
use omnix_common::config::OmConfig;

use crate::readme::Readme;

#[derive(Debug, Deserialize, Clone, Default)]
pub struct HackConfig {
    pub readme: Readme,
}

#[derive(Debug, Deserialize, Clone)]
pub struct CacheConfig {
    /// Cache substituter URL
    pub url: String,
}

impl HackConfig {
    pub async fn from_flake(url: &FlakeUrl) -> anyhow::Result<Self> {
        let v = OmConfig::<Self>::from_flake_url(NixCmd::get().await, url, &["om.hack"])
            .await?
            .config;
        let config = v.get("default").cloned().unwrap_or_default();
        Ok(config)
    }
}
