//! Nix flake outputs

use serde::{Deserialize, Serialize};
use std::{borrow::Borrow, collections::HashMap};

use super::schema::Val;

/// Flake outputs derived from [super::schema::FlakeSchemas]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum FlakeOutputs {
    /// A terminal value of a flake output
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
    /// let tree = nix_rs::flake::outputs::FlakeOutputs::default();
    /// let val = tree.get(&["aarch64-darwin", "default"]);
    /// ```
    pub fn get<Q>(&self, path: &[&Q]) -> Option<&Self>
    where
        Q: ?Sized + Eq + std::hash::Hash,
        String: Borrow<Q>,
    {
        let mut current = self;
        for key in path {
            match current {
                Self::Attrset(map) => {
                    current = map.get(key.borrow())?;
                }
                Self::Val(_) => return None,
            }
        }
        Some(current)
    }
}
