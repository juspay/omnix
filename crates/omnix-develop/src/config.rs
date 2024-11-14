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
        let (config, _rest) = om_config.get_sub_config_under("develop")?;
        Ok(config)
    }
}
