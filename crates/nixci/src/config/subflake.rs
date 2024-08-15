use std::collections::BTreeMap;

use nix_rs::flake::{system::System, url::FlakeUrl};
use serde::Deserialize;

use crate::step::core::Steps;

/// Represents a sub-flake look-alike.
///
/// "Look-alike" because its inputs may be partial, thus requiring explicit
/// --override-inputs when evaluating the flake.
#[derive(Debug, Deserialize)]
pub struct SubflakeConfig {
    /// Whether to skip building this subflake
    #[serde(default)]
    pub skip: bool,

    /// Subdirectory in which the flake lives
    pub dir: String,

    /// Inputs to override (via --override-input)
    // NB: we use BTreeMap instead of HashMap here so that we always iterate
    // inputs in a determinitstic (i.e. asciibetical) order
    #[serde(rename = "overrideInputs", default)]
    pub override_inputs: BTreeMap<String, FlakeUrl>,

    /// An optional whitelist of systems to build on (others are ignored)
    pub systems: Option<Vec<System>>,

    /// List of CI steps to run
    #[serde(default)]
    pub steps: Steps,
}

impl Default for SubflakeConfig {
    /// The default `SubflakeConfig` is the root flake.
    fn default() -> Self {
        SubflakeConfig {
            skip: false,
            dir: ".".to_string(),
            override_inputs: BTreeMap::default(),
            systems: None,
            steps: Steps::default(),
        }
    }
}

impl SubflakeConfig {
    pub fn can_build_on(&self, systems: &[System]) -> bool {
        match self.systems.as_ref() {
            Some(systems_whitelist) => systems_whitelist.iter().any(|s| systems.contains(s)),
            None => true,
        }
    }
}
