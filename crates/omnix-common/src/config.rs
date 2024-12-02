//! Manage omnix configuration in flake.nix

use std::collections::BTreeMap;

use nix_rs::{
    command::NixCmd,
    flake::{eval::nix_eval_attr, metadata::FlakeMetadata, url::FlakeUrl},
};
use serde::{de::DeserializeOwned, Deserialize};
#[cfg(test)]
use std::str::FromStr;

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
    /// Fetch the `om` configuration from `om.yaml` if present, falling back to `om` config in flake output
    pub async fn get(flake_url: &FlakeUrl) -> Result<Self, OmConfigError> {
        match Self::from_yaml(flake_url).await? {
            None => Self::from_flake(flake_url).await,
            Some(config) => Ok(config),
        }
    }

    /// Read the configuration from `om.yaml` in flake root
    async fn from_yaml(flake_url: &FlakeUrl) -> Result<Option<Self>, OmConfigError> {
        let path = if let Some(local_path) = flake_url.without_attr().as_local_path() {
            local_path.to_path_buf()
        } else {
            FlakeMetadata::from_nix(NixCmd::get().await, &flake_url.without_attr())
                .await?
                .path
        }
        .join("om.yaml");

        if !path.exists() {
            return Ok(None);
        }

        let yaml_str = std::fs::read_to_string(path)?;
        let config: OmConfigTree = serde_yaml::from_str(&yaml_str)?;
        Ok(Some(OmConfig {
            flake_url: flake_url.without_attr(),
            reference: flake_url.get_attr().as_list(),
            config,
        }))
    }

    /// Read the configuration from `om` flake output
    async fn from_flake(flake_url: &FlakeUrl) -> Result<Self, OmConfigError> {
        Ok(OmConfig {
            flake_url: flake_url.without_attr(),
            reference: flake_url.get_attr().as_list(),
            config: nix_eval_attr(NixCmd::get().await, &flake_url.with_attr("om"))
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
            None => return Ok((T::default(), &[])),
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

    /// A [nix_rs::command::NixCmdError]
    #[error("Nix command error: {0}")]
    NixCmdError(#[from] nix_rs::command::NixCmdError),

    /// Failed to parse JSON
    #[error("Failed to decode (json error): {0}")]
    DecodeErrorJson(#[from] serde_json::Error),

    /// Failed to parse yaml
    #[error("Failed to parse yaml: {0}")]
    ParseYaml(#[from] serde_yaml::Error),

    /// Failed to read yaml
    #[error("Failed to read yaml: {0}")]
    ReadYaml(#[from] std::io::Error),
}

#[tokio::test]
async fn test_get_missing_sub_config() {
    let om_config_empty_reference = OmConfig {
        flake_url: FlakeUrl::from_str(".").unwrap(),
        reference: vec![],
        config: serde_yaml::from_str("").unwrap(),
    };
    let om_config_with_reference = OmConfig {
        flake_url: FlakeUrl::from_str(".").unwrap(),
        reference: vec!["foo".to_string()],
        config: serde_yaml::from_str("").unwrap(),
    };

    let (res_empty_reference, _rest) = om_config_empty_reference
        .get_sub_config_under::<String>("health")
        .unwrap();
    let (res_with_reference, _rest) = om_config_with_reference
        .get_sub_config_under::<String>("health")
        .unwrap();

    assert_eq!(res_empty_reference, String::default());
    assert_eq!(res_with_reference, String::default());
}

#[tokio::test]
async fn test_get_omconfig_from_remote_flake_with_attr() {
    let om_config = OmConfig::get(
        &FlakeUrl::from_str(
            "github:juspay/omnix/0ed2a389d6b4c8eb78caed778e20e872d2a59973#default.omnix",
        )
        .unwrap(),
    )
    .await;
    assert!(om_config.is_ok());
}
