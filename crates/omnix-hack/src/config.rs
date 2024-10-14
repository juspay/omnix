use nix_rs::{
    command::NixCmd,
    flake::{command::FlakeOptions, url::FlakeUrl},
};
use omnix_common::config::OmConfig;
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct HackConfig {
    pub cache: CacheConfig,
    pub readme: ReadmeConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct CacheConfig {
    pub cachix: CachixConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct CachixConfig {
    pub enable: bool,
    pub name: String,
    /// The read-only auth token to use if a private cache
    pub auth_token: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ReadmeConfig(pub String);

impl HackConfig {
    pub async fn from_flake(url: &FlakeUrl) -> anyhow::Result<Self> {
        let _opts = FlakeOptions {
            refresh: true,
            ..Default::default()
        };
        let v = OmConfig::<Self>::from_flake_url(NixCmd::get().await, url, &["om.hack"])
            .await?
            .config;
        v.get("default")
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("Missing key default for om.hack"))
    }
}
