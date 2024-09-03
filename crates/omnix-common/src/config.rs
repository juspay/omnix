//! Manage omnix configuration in flake.nix

use std::collections::BTreeMap;

use nix_rs::{
    command::{NixCmd, NixCmdError},
    flake::{
        outputs::{FilteredFlakeOutputs, QualifiedAttrError},
        url::FlakeUrl,
    },
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
    /// Read the Om configuration from [FilteredFlakeOutputs]
    pub async fn from_flake_outputs<S>(
        cmd: &NixCmd,
        url: &FlakeUrl,
        k: &[S],
    ) -> Result<OmConfig<T>, OmConfigError>
    where
        S: AsRef<str>,
        T: Default + DeserializeOwned + std::fmt::Debug,
    {
        let filtered_outputs = FilteredFlakeOutputs::from_nix(cmd, &url.without_attr()).await?;
        let (config, reference) = filtered_outputs.find_qualified_attr(url, k).await?;
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

    /// Nix command error
    #[error("Nix command error: {0}")]
    NixCmdError(#[from] NixCmdError),
}
