//! Manage omnix configuration in flake.nix

use std::collections::BTreeMap;

use nix_rs::{
    command::NixCmd,
    flake::{eval::nix_eval_attr, url::FlakeUrl},
};
use serde::de::DeserializeOwned;

/// Reference to the whole `om` configuration in a flake
#[derive(Debug)]
pub struct OmConfig {
    /// The flake URL used to load this configuration
    pub flake_url: FlakeUrl,

    /// The (nested) key reference into the flake config.
    pub reference: Vec<String>,

    /// The whole `om` configuration parsed from JSON
    pub config: BTreeMap<String, BTreeMap<String, serde_json::Value>>,
}

impl OmConfig {
    /// Read the om configuration from the flake url
    pub async fn from_flake_url(cmd: &NixCmd, flake_url: &FlakeUrl) -> Result<Self, OmConfigError> {
        let qualified_url = flake_url.with_attr("om");
        let config = nix_eval_attr(cmd, &qualified_url)
            .await?
            .unwrap_or_default();

        Ok(OmConfig {
            flake_url: flake_url.clone(),
            reference: flake_url.get_attr().as_list(),
            config,
        })
    }

    /// Get all the configs of type `T` for a given sub-config
    pub fn get_sub_configs<T>(
        &self,
        sub_config_name: &str,
    ) -> Result<BTreeMap<String, T>, OmConfigError>
    where
        T: DeserializeOwned,
    {
        let config = self
            .config
            .get(sub_config_name)
            .ok_or_else(|| OmConfigError::MissingConfigAttribute(sub_config_name.to_string()))?;
        config
            .iter()
            .map(|(k, v)| {
                serde_json::from_value(v.clone())
                    .map_err(OmConfigError::from)
                    .map(|converted| (k.clone(), converted))
            })
            .collect()
    }

    /// Get the user-referenced config value `T` for a given sub-config
    pub fn get_referenced_for<T>(&self, sub_config: &str) -> Result<(T, &[String]), OmConfigError>
    where
        T: Default + DeserializeOwned,
    {
        // Early return if sub_config doesn't exist
        let config = match self.config.get(sub_config) {
            Some(config) => config,
            None => {
                return match self.flake_url.get_attr().0 {
                    None => Ok((T::default(), &[])),
                    Some(attr) => Err(OmConfigError::UnexpectedAttribute(attr)),
                }
            }
        };

        // Try to get value from reference path first
        if let Some((k, rest)) = self.reference.split_first() {
            let value = config
                .get(k)
                .ok_or_else(|| OmConfigError::MissingConfigAttribute(k.to_string()))?;
            return Ok((serde_json::from_value(value.clone())?, rest));
        }

        // Fall back to default or T::default()
        let value = config
            .get("default")
            .map(|v| serde_json::from_value(v.clone()))
            .transpose()?
            .unwrap_or_default();

        Ok((value, &[]))
    }
}

/// Error type for OmConfig
#[derive(thiserror::Error, Debug)]
pub enum OmConfigError {
    /// Missing configuration attribute
    #[error("Missing configuration attribute: {0}")]
    MissingConfigAttribute(String),

    /// Unexpected attribute
    #[error("Unexpected attribute: {0}")]
    UnexpectedAttribute(String),

    /// A [nix_rs::command::NixCmdError]
    #[error("Nix command error: {0}")]
    NixCmdError(#[from] nix_rs::command::NixCmdError),

    /// Failed to parse JSON
    #[error("Failed to decode (json error): {0}")]
    DecodeErrorJson(#[from] serde_json::Error),
}
