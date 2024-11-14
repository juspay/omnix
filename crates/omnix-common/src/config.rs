//! Manage omnix configuration in flake.nix

use std::collections::BTreeMap;

use nix_rs::{
    command::NixCmd,
    flake::{eval::nix_eval_attr, url::FlakeUrl},
};
use serde::{de::DeserializeOwned, Deserialize};

/// [Config] with additional metadata about the flake URL and reference.
///
/// `reference` here is the part of the flake URL after `#`
#[derive(Debug)]
pub struct OmConfig {
    /// The flake URL used to load this configuration
    pub flake_url: FlakeUrl,

    /// The (nested) key reference into the flake config.
    pub reference: Vec<String>,

    /// omnix [Config]
    pub config: Config,
}

impl OmConfig {
    /// Read the om configuration from the flake url
    pub async fn from_flake_url(cmd: &NixCmd, flake_url: &FlakeUrl) -> Result<Self, OmConfigError> {
        Ok(OmConfig {
            flake_url: flake_url.clone(),
            reference: flake_url.get_attr().as_list(),
            config: nix_eval_attr(cmd, &flake_url.with_attr("om"))
                .await?
                .unwrap_or_default(),
        })
    }

    /// Get the user-referenced config value `T` for a given sub-config
    pub fn get_referenced_for<T>(&self, sub_config: &str) -> Result<(T, &[String]), OmConfigError>
    where
        T: Default + DeserializeOwned,
    {
        // Get the config map, returning default if it doesn't exist
        let config = match self.config.get::<T>(sub_config) {
            Some(Ok(config)) => config,
            Some(Err(e)) => return Err(OmConfigError::DecodeErrorJson(e)),
            None => {
                return match self.flake_url.get_attr().0 {
                    None => Ok((T::default(), &[])),
                    Some(attr) => Err(OmConfigError::UnexpectedAttribute(attr)),
                }
            }
        };

        // Try to get value from reference path first
        if let Some((k, rest)) = self.reference.split_first() {
            return config
                .into_iter()
                .find(|(cfg_name, _)| cfg_name == k)
                .map(|(_, v)| (v, rest))
                .ok_or_else(|| OmConfigError::MissingConfigAttribute(k.to_string()));
        }

        // Fall back to `default` attribute or `T::default()`
        Ok((
            config
                .into_iter()
                .find_map(|(k, v)| if k == "default" { Some(v) } else { None })
                .unwrap_or_default(),
            &[],
        ))
    }
}

/// Represents the whole configuration for `omnix` parsed from JSON
#[derive(Debug, Default, Deserialize)]
pub struct Config(BTreeMap<String, BTreeMap<String, serde_json::Value>>);

impl Config {
    /// Get all the configs of type `T` for a given sub-config
    /// Returns None if sub_config doesn't exist, or Some(Err) if deserialization fails
    pub fn get<T>(&self, sub_config: &str) -> Option<Result<BTreeMap<String, T>, serde_json::Error>>
    where
        T: DeserializeOwned,
    {
        self.0.get(sub_config).map(|config| {
            config
                .iter()
                .map(|(k, v)| serde_json::from_value(v.clone()).map(|value| (k.clone(), value)))
                .collect()
        })
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
