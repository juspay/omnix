use serde::Deserialize;

use nix_rs::{command::NixCmd, flake::url::FlakeUrl};
use omnix_common::config::{OmConfig, OmnixConfig};

use crate::readme::Readme;

#[derive(Debug, Deserialize, Clone, Default)]
pub struct DevelopConfig {
    pub readme: Readme,
}

#[derive(Debug, Deserialize, Clone)]
pub struct CacheConfig {
    /// Cache substituter URL
    pub url: String,
}

impl DevelopConfig {
    pub async fn from_om_config(om_config: &OmnixConfig) -> anyhow::Result<Self> {
        if let Some(v) = om_config.config.get("develop") {
            let config = v.get("default").cloned().unwrap_or_default();
            let v1 = serde_json::from_value(config)?;
            Ok(v1)
        } else {
            Ok(Default::default())
        }
    }
    pub async fn from_flake(url: &FlakeUrl) -> anyhow::Result<Self> {
        let v = OmConfig::<Self>::from_flake_url(NixCmd::get().await, url, &["om.develop"])
            .await?
            .config;
        let config = v.get("default").cloned().unwrap_or_default();
        Ok(config)
    }
}
