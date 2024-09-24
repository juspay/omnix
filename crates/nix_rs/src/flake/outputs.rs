//! Nix flake outputs

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::schema::{FlakeSchemas, Val};

/// Flake outputs derived from [super::schema::FlakeSchemas]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum FlakeOutputs {
    #[allow(missing_docs)]
    Val(Val),
    /// A tree-like structure representing a flake output.
    /// Each key in the map represents a top-level flake output.
    Attrset(HashMap<String, FlakeOutputs>),
}

impl FlakeOutputs {
    /// Get the non-attrset value
    pub fn get_val(&self) -> Option<&Val> {
        match self {
            Self::Val(v) => Some(v),
            _ => None,
        }
    }

    /// Get the attrset as a vector of key-value pairs
    pub fn get_children(&self) -> Vec<(String, Val)> {
        match self {
            Self::Val(_) => vec![],
            Self::Attrset(map) => map
                .iter()
                .filter_map(|(k, v)| v.get_val().map(|val| (k.clone(), val.clone())))
                .collect(),
        }
    }

    /// Lookup the given path, returning a reference to the value if it exists.
    ///
    /// # Example
    /// ```no_run
    /// let tree : &nix_rs::flake::outputs::FlakeOutputs = todo!();
    /// let val = tree.get(&["aarch64-darwin", "default"]);
    /// ```
    pub fn get(&self, path: &[&str]) -> Option<&Self> {
        let mut current = self;
        for key in path {
            match current {
                Self::Attrset(map) => {
                    current = map.get(*key)?;
                }
                Self::Val(_) => return None,
            }
        }
        Some(current)
    }
}

impl From<FlakeSchemas> for FlakeOutputs {
    fn from(schema: FlakeSchemas) -> Self {
        schema.to_flake_outputs()
    }
}
