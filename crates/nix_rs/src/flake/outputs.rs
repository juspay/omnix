//! Nix flake outputs
// TODO: Document this module!
#![allow(missing_docs)]

use serde::{Deserialize, Serialize};
use std::collections::{btree_map::Entry, BTreeMap};

use super::schema::Val;

/// Flake outputs derived from [super::schema::FlakeSchemas]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum FlakeOutputs {
    Val(Val),
    Attrset(BTreeMap<String, FlakeOutputs>),
}

impl FlakeOutputs {
    /// Get the non-attrset value
    pub fn as_val(&self) -> Option<&Val> {
        match self {
            Self::Val(v) => Some(v),
            _ => None,
        }
    }

    /// Get the attrset as a vector of key-value pairs
    pub fn as_vec(self) -> Vec<(String, Val)> {
        match self {
            Self::Val(_) => vec![],
            Self::Attrset(map) => map
                .into_iter()
                .filter_map(|(k, v)| v.as_val().map(|val| (k, val.clone())))
                .collect(),
        }
    }

    /// Lookup the given path, returning the value, while removing it from the tree.
    ///
    /// # Example
    /// ```no_run
    /// let tree : &nix_rs::flake::outputs::FlakeOutputs = todo!();
    /// let val = tree.pop(&["aarch64-darwin", "default"]);
    /// ```
    pub fn pop(&mut self, path: &[&str]) -> Option<Self> {
        let mut curr = self;
        let mut path = path.iter().peekable();
        while let Some(part) = path.next() {
            let Self::Attrset(v) = curr else {
                return None;
            };
            let Entry::Occupied(entry) = v.entry(part.to_string()) else {
                return None;
            };
            if path.peek().is_none() {
                return Some(entry.remove());
            } else {
                curr = entry.into_mut();
            }
        }
        None
    }
}
