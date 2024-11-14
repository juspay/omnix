//! Manage omnix configuration in flake.nix

use std::collections::BTreeMap;

use nix_rs::{
    command::NixCmd,
    flake::{eval::nix_eval_attr, url::FlakeUrl},
};
use serde::{de::DeserializeOwned, Deserialize};

/// [OmConfigTree] with additional metadata about the flake URL and reference.
///
/// `reference` here is the part of the flake URL after `#`
#[derive(Debug)]
pub struct OmConfig {
    /// The flake URL used to load this configuration
    pub flake_url: FlakeUrl,

    /// The (nested) key reference into the flake config.
    pub reference: Vec<String>,

    /// The config tree
    pub config: OmConfigTree,
}

impl OmConfig {
    /// Read the om configuration from the flake url
    pub async fn from_flake_url(cmd: &NixCmd, flake_url: &FlakeUrl) -> Result<Self, OmConfigError> {
        Ok(OmConfig {
            flake_url: flake_url.without_attr(),
            reference: flake_url.get_attr().as_list(),
            config: nix_eval_attr(cmd, &flake_url.with_attr("om"))
                .await?
                .unwrap_or_default(),
        })
    }

    /// Get the user referenced (per `referenced`) sub-tree under the given root key.
    ///
    /// get_sub_config_under("ci") will return `ci.default` (or Default instance if config is missing) without a reference. Otherwise, it will use the reference to find the correct sub-tree.
    pub fn get_sub_config_under<T>(&self, root_key: &str) -> Result<(T, &[String]), OmConfigError>
    where
        T: Default + DeserializeOwned + Clone,
    {
        // Get the config map, returning default if it doesn't exist
        let config = match self.config.get::<T>(root_key)? {
            Some(res) => res,
            None => {
                return if self.reference.is_empty() {
                    Ok((T::default(), &[]))
                } else {
                    // Reference requires the config to exist.
                    Err(OmConfigError::UnexpectedAttribute(self.reference.join(".")))
                };
            }
        };

        let default = "default".to_string();
        let (k, rest) = self.reference.split_first().unwrap_or((&default, &[]));

        let v: &T = config
            .get(k)
            .ok_or(OmConfigError::MissingConfigAttribute(k.to_string()))?;
        Ok((v.clone(), rest))
    }
}

/// Represents the whole configuration for `omnix` parsed from JSON
#[derive(Debug, Default, Deserialize)]
pub struct OmConfigTree(BTreeMap<String, BTreeMap<String, serde_json::Value>>);

impl OmConfigTree {
    /// Get all the configs of type `T` for a given sub-config
    ///
    /// Return None if key doesn't exist
    pub fn get<T>(&self, key: &str) -> Result<Option<BTreeMap<String, T>>, serde_json::Error>
    where
        T: DeserializeOwned,
    {
        match self.0.get(key) {
            Some(config) => {
                let result: Result<BTreeMap<String, T>, _> = config
                    .iter()
                    .map(|(k, v)| serde_json::from_value(v.clone()).map(|value| (k.clone(), value)))
                    .collect();
                result.map(Some)
            }
            None => Ok(None),
        }
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
