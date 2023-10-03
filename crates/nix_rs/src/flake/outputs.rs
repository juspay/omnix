//! Nix flake outputs

use serde::{Deserialize, Serialize};
use std::collections::{btree_map::Entry, BTreeMap};

/// Represents the "outputs" of a flake
///
/// This structure is currently produced by `nix flake show`
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum FlakeOutputs {
    Val(Val),
    Attrset(BTreeMap<String, FlakeOutputs>),
}

impl FlakeOutputs {
    /// Run `nix flake show` on the given flake url
    #[cfg(feature = "ssr")]
    #[tracing::instrument(name = "flake-show")]
    pub async fn from_nix(
        nix_cmd: &crate::command::NixCmd,
        flake_url: &super::url::FlakeUrl,
    ) -> Result<Self, crate::command::NixCmdError> {
        let v = nix_cmd
            .run_with_args_expecting_json(&[
                "flake",
                "show",
                "--legacy", // for showing nixpkgs legacyPackages
                "--allow-import-from-derivation",
                "--json",
                &flake_url.to_string(),
            ])
            .await?;
        Ok(v)
    }

    /// Get the non-attrset value
    pub fn as_leaf(&self) -> Option<&Val> {
        match self {
            Self::Val(v) => Some(v),
            _ => None,
        }
    }

    /// Ensure the value is an attrset, and get it
    pub fn as_attrset(&self) -> Option<&BTreeMap<String, FlakeOutputs>> {
        match self {
            Self::Attrset(v) => Some(v),
            _ => None,
        }
    }

    /// Lookup the given path, returning the value, while removing it from the tree.
    ///
    /// # Example
    /// ```no_run
    /// let tree : &nix_rs::flake::outputs::FlakeOutputs = todo!();
    /// let val = tree.pop(&["packages", "aarch64-darwin", "default"]);
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

/// The metadata of a flake output value which is of non-attrset [Type]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Val {
    #[serde(rename = "type")]
    pub type_: Type,
    pub name: Option<String>,
    pub description: Option<String>,
}

/// The type of a flake output [Val]
///
/// [Nix source ref](https://github.com/NixOS/nix/blob/2.14.1/src/nix/flake.cc#L1105)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Type {
    NixosModule,
    Derivation,
    App,
    Template,
    #[serde(other)]
    Unknown,
}

impl Type {
    /// Get the icon for this type
    pub fn to_icon(&self) -> &'static str {
        match self {
            Self::NixosModule => "❄️",
            Self::Derivation => "📦",
            Self::App => "📱",
            Self::Template => "🏗️",
            Self::Unknown => "❓",
        }
    }
}
