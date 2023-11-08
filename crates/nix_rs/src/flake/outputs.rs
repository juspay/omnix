//! Nix flake outputs

use serde::{Deserialize, Serialize};
use std::{
    collections::{btree_map::Entry, BTreeMap},
    fmt::Display,
};

/// Represents the "outputs" of a flake
///
/// This structure is currently produced by `nix flake show`, thus to parse it we must toggle serde untagged.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FlakeOutputs {
    Val(Val),
    Attrset(BTreeMap<String, FlakeOutputs>),
}

impl FlakeOutputs {
    /// Run `nix flake show` on the given flake url
    pub async fn from_nix(
        nix_cmd: &crate::command::NixCmd,
        flake_url: &super::url::FlakeUrl,
    ) -> Result<Self, crate::command::NixCmdError> {
        let v = FlakeOutputsUntagged::from_nix(nix_cmd, flake_url).await?;
        Ok(v.into_flake_outputs())
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

impl Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!("{:?}", self))
    }
}

/// This type is identical to [FlakeOutputs] except for the serde untagged attribute, which enables parsing the JSON output of `nix flake show`.
///
/// This separation exists to workaround https://github.com/DioxusLabs/dioxus-std/issues/20
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
enum FlakeOutputsUntagged {
    UVal(Val),
    UAttrset(BTreeMap<String, FlakeOutputsUntagged>),
}

impl FlakeOutputsUntagged {
    /// Run `nix flake show` on the given flake url
    #[tracing::instrument(name = "flake-show")]
    async fn from_nix(
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

    /// Convert to [FlakeOutputs]
    fn into_flake_outputs(self) -> FlakeOutputs {
        match self {
            Self::UVal(v) => FlakeOutputs::Val(v),
            Self::UAttrset(v) => FlakeOutputs::Attrset(
                v.into_iter()
                    .map(|(k, v)| (k, v.into_flake_outputs()))
                    .collect(),
            ),
        }
    }
}
