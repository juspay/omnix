//! Manage omnix configuration in flake.nix

use std::collections::BTreeMap;

use nix_rs::{
    command::NixCmd,
    flake::{
        eval::nix_eval_attr,
        url::{
            qualified_attr::{nix_eval_qualified_attr, QualifiedAttrError},
            FlakeUrl,
        },
    },
};
use serde::de::DeserializeOwned;

/// Reference to the whole `om` configuration in a flake
pub struct OmnixConfig {
    /// The flake URL used to load this configuration
    pub flake_url: FlakeUrl,

    /// The (nested) key reference into the flake config.
    pub reference: Vec<String>,

    /// The whole `om` configuration parsed from JSON
    pub config: BTreeMap<String, BTreeMap<String, serde_json::Value>>,
}

impl OmnixConfig {
    /// Read the om configuration from the flake url
    pub async fn from_flake_url(cmd: &NixCmd, flake_url: &FlakeUrl) -> Result<Self, OmConfigError> {
        let qualified_url = flake_url.with_attr("om");
        let config = nix_eval_attr(cmd, &qualified_url)
            .await?
            .unwrap_or_default();

        Ok(OmnixConfig {
            flake_url: flake_url.clone(),
            reference: flake_url.get_attr().as_list(),
            config,
        })
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
    /// Read the Om configuration from the flake URL
    pub async fn from_flake_url<S>(
        cmd: &NixCmd,
        url: &FlakeUrl,
        k: &[S],
    ) -> Result<OmConfig<T>, OmConfigError>
    where
        S: AsRef<str>,
        T: DeserializeOwned,
    {
        let (config, reference) =
            nix_eval_qualified_attr::<BTreeMap<String, T>, _>(cmd, url, k).await?;
        Ok(OmConfig {
            flake_url: url.without_attr(),
            reference,
            config,
        })
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
    /// Qualified attribute error
    #[error("Qualified attribute error: {0}")]
    QualifiedAttrError(#[from] QualifiedAttrError),

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
