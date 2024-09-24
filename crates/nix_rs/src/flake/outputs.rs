//! Nix flake outputs

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::schema::{FlakeSchemas, Val};

/// Outputs of a flake
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum FlakeOutputs {
    /// Terminal value that is not an attrset.
    Val(Val),
    /// An attrset of nested [FlakeOutputs]
    Attrset(HashMap<String, FlakeOutputs>),
}

impl FlakeOutputs {
    /// Get the terminal value
    pub fn get_val(&self) -> Option<&Val> {
        match self {
            Self::Val(v) => Some(v),
            _ => None,
        }
    }

    /// Get the attrset
    pub fn get_attrset(&self) -> Option<&HashMap<String, FlakeOutputs>> {
        match self {
            Self::Val(_) => None,
            Self::Attrset(map) => Some(map),
        }
    }

    /// Get the attrset as a vector of key-value pairs
    ///
    /// **NOTE**: Only terminal values are included!
    pub fn get_attrset_of_val(&self) -> Vec<(String, Val)> {
        self.get_attrset().map_or(vec![], |map| {
            map.iter()
                .filter_map(|(k, v)| v.get_val().map(|val| (k.clone(), val.clone())))
                .collect()
        })
    }

    /// Lookup the given path, returning a reference to the value if it exists.
    ///
    /// # Example
    /// ```no_run
    /// let tree : &nix_rs::flake::outputs::FlakeOutputs = todo!();
    /// let val = tree.get_by_path(&["aarch64-darwin", "default"]);
    /// ```
    pub fn get_by_path(&self, path: &[&str]) -> Option<&Self> {
        let mut current = self;
        for key in path {
            let map = current.get_attrset()?;
            current = map.get(*key)?;
        }
        Some(current)
    }
}

impl From<FlakeSchemas> for FlakeOutputs {
    fn from(schema: FlakeSchemas) -> Self {
        schema.to_flake_outputs()
    }
}
