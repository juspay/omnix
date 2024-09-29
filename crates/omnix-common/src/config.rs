//! Manage omnix configuration in flake.nix

use std::collections::BTreeMap;

use nix_rs::{
    command::{NixCmd, NixCmdError},
    config::{NixConfig, NixConfigError},
    flake::{outputs::FlakeOutputs, schema::FlakeSchemas, url::FlakeUrl},
};
use serde::de::DeserializeOwned;

/// Reference to some Omnix configuration of type `BTeeMap<String, T>` in a flake
///
/// For example, CI configuration at `om.ci.default` is captured by the `T` type.
#[derive(Debug)]
pub struct OmConfig<T> {
    /// The flake URL used to load this configuration
    pub flake_url: FlakeUrl,

    /// The (nested) key reference into the flake config.
    pub reference: Vec<String>,

    /// The whole `om.??` configuration parsed as `T`
    pub config: BTreeMap<String, T>,
}

impl<T> OmConfig<T> {
    /// Read the Om configuration from a flake
    /// TODO: simplify the implementation, it has too many responsibilities
    pub async fn from_nix(
        cmd: &NixCmd,
        url: &FlakeUrl,
        k: &[&[&str]],
    ) -> Result<OmConfig<T>, OmConfigError>
    where
        T: Default + DeserializeOwned + std::fmt::Debug,
    {
        let nix_config = NixConfig::get().await.as_ref().unwrap();
        let schema = FlakeSchemas::from_nix(cmd, url, &nix_config.system.value).await?;
        let outputs = FlakeOutputs::from(schema);

        let cfg = outputs.get_first_by_paths(k);
        match cfg {
            Some(cfg) => Ok(OmConfig {
                flake_url: url.without_attr(),
                reference: url.get_attr().as_list(),
                config: cfg.deserialize()?,
            }),
            None => match url.get_attr().0 {
                None => Ok(OmConfig {
                    flake_url: url.without_attr(),
                    reference: vec![],
                    config: Default::default(),
                }),
                Some(attr) => Err(OmConfigError::MissingConfigAttribute(attr)),
            },
        }
    }

    /// Get the user-referenced config value `T`
    ///
    /// If the user passes `.#foo.bar` this selects "foo" from the config tree, along with returning  ["bar"].
    ///
    /// If nothing is specifically passed, a default value is returned, either from config tree (key "default") or `T::default()`.
    ///
    /// TODO: This needs to be adjusted to support `om.templates` style configuration as well, where this default behaviour makes no sense.
    pub fn get_referenced(&self) -> Result<(T, &[String]), OmConfigError>
    where
        T: Default + Clone,
    {
        if let Some((k, rest)) = self.reference.split_first() {
            if let Some(v) = self.config.get(k) {
                Ok((v.clone(), rest))
            } else {
                Err(OmConfigError::MissingConfigAttribute(k.to_string()))
            }
        } else {
            // Use default
            if let Some(v) = self.config.get("default") {
                Ok((v.clone(), &[]))
            } else {
                Ok((T::default(), &[]))
            }
        }
    }
}

/// Error type for OmConfig
#[derive(thiserror::Error, Debug)]
pub enum OmConfigError {
    /// Serde JSON error
    #[error("Serde JSON error: {0}")]
    SerdeJson(#[from] serde_json::Error),

    /// Missing configuration attribute
    #[error("Missing configuration attribute: {0}")]
    MissingConfigAttribute(String),

    /// Nix command error
    #[error("Nix command error: {0}")]
    NixCmd(#[from] NixCmdError),

    /// Nix config error
    #[error("Nix config error: {0}")]
    NixConfig(#[from] NixConfigError),
}
