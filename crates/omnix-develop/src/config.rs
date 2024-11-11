use serde::Deserialize;

use omnix_common::config::OmConfig;

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
    pub fn from_om_config(om_config: &OmConfig) -> anyhow::Result<Self> {
        if let Some(v) = om_config.config.get("develop") {
            let config = v.get("default").cloned().unwrap_or_default();
            let v1 = serde_json::from_value(config)?;
            Ok(v1)
        } else {
            Ok(Default::default())
        }
    }
}
