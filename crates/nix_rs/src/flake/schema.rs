//! Nix flake-schemas

use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, HashMap},
    fmt::Display,
    path::Path,
};

use crate::system_list::SystemsListFlakeRef;

use super::{command::FlakeOptions, eval::nix_eval, outputs::FlakeOutputs, url::FlakeUrl};

lazy_static! {
  /// Flake URL of the default flake schemas
  ///
  /// We expect this environment to be set in Nix build and shell.
  pub static ref DEFAULT_FLAKE_SCHEMAS: FlakeUrl = {
    Into::<FlakeUrl>::into(Path::new(env!("DEFAULT_FLAKE_SCHEMAS")))
  };

  /// Flake URL of the flake that defines functions for inspecting flake outputs
  ///
  /// We expect this environment to be set in Nix build and shell.
  pub static ref INSPECT_FLAKE: FlakeUrl = {
    Into::<FlakeUrl>::into(Path::new(env!("INSPECT_FLAKE")))
  };
}

/// Represents the schema of a given flake evaluated using [static@INSPECT_FLAKE]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FlakeSchemas {
    /// Each key in the map represents either a top-level flake output or other metadata (e.g. `docs`)
    pub inventory: HashMap<String, InventoryItem>,
}

/// A tree-like structure representing each flake output or metadata in [FlakeSchemas]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum InventoryItem {
    /// Represents a terminal node in the tree
    Leaf(Leaf),
    /// Represents a non-terminal node in the tree
    Attrset(HashMap<String, InventoryItem>),
}

impl FlakeSchemas {
    /// Get the [FlakeSchema] for the given flake
    ///
    /// This uses [static@INSPECT_FLAKE] and [static@DEFAULT_FLAKE_SCHEMAS]
    pub async fn from_nix(
        nix_cmd: &crate::command::NixCmd,
        flake_url: &super::url::FlakeUrl,
        system: &super::System,
    ) -> Result<Self, crate::command::NixCmdError> {
        let inspect_flake: FlakeUrl = INSPECT_FLAKE
            // Why `exculdingOutputPaths`?
            //   This function is much faster than `includingOutputPaths` and also solves <https://github.com/juspay/omnix/discussions/231>
            //   Also See: https://github.com/DeterminateSystems/inspect/blob/7f0275abbdc46b3487ca69e2acd932ce666a03ff/flake.nix#L139
            //
            //
            // Note: We might need to use `includingOutputPaths` in the future, when replacing `devour-flake`.
            // In which case, `om ci` and `om show` can invoke the appropriate function from `INSPECT_FLAKE`.
            //
            .with_attr("contents.excludingOutputPaths");
        let systems_flake = SystemsListFlakeRef::from_known_system(system)
            // TODO: don't use unwrap
            .unwrap()
            .0
            .clone();
        let flake_opts = FlakeOptions {
            no_write_lock_file: true,
            override_inputs: BTreeMap::from_iter([
                (
                    "flake-schemas".to_string(),
                    DEFAULT_FLAKE_SCHEMAS.to_owned(),
                ),
                ("flake".to_string(), flake_url.clone()),
                ("systems".to_string(), systems_flake),
            ]),
            ..Default::default()
        };
        let v = nix_eval::<Self>(nix_cmd, &flake_opts, &inspect_flake).await?;
        Ok(v)
    }

    /// Convert [FlakeSchemas] to [FlakeOutputs]
    pub fn to_flake_outputs(&self) -> FlakeOutputs {
        FlakeOutputs::Attrset(
            self.inventory
                .iter()
                .filter_map(|(k, v)| Some((k.clone(), v.to_flake_outputs()?)))
                .collect(),
        )
    }
}

impl InventoryItem {
    fn to_flake_outputs(&self) -> Option<FlakeOutputs> {
        match self {
            Self::Leaf(Leaf::Val(v)) => Some(FlakeOutputs::Val(v.clone())),
            Self::Attrset(map) => {
                if let Some(children) = map.get("children") {
                    children.to_flake_outputs()
                } else {
                    let filtered: HashMap<_, _> = map
                        .iter()
                        .filter_map(|(k, v)| Some((k.clone(), v.to_flake_outputs()?)))
                        .collect();
                    if filtered.is_empty() {
                        None
                    } else {
                        Some(FlakeOutputs::Attrset(filtered))
                    }
                }
            }
            _ => None,
        }
    }
}

/// Represents a leaf value of a flake output
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Leaf {
    /// A terminal value of a flake output
    Val(Val),
    /// Represents description for a flake output
    /// (e.g. `Doc` for `formatter` will be "The `formatter` output specifies the package to use to format the project.")
    Doc(String),
}

/// A terminal value of a flake output
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Val {
    #[serde(rename = "what")]
    /// Represents the type of the flake output
    pub type_: Type,
    /// If the flake output is a derivation, this will be the name of the derivation
    pub derivation_name: Option<String>,
    /// A short description derived from `meta.description` of the derivation with [Val::derivation_name]
    pub short_description: Option<String>,
}

impl Default for Val {
    fn default() -> Self {
        Self {
            type_: Type::Unknown,
            derivation_name: None,
            short_description: None,
        }
    }
}

/// The type of a flake output [Val]
///
/// These types can differ based on [static@DEFAULT_FLAKE_SCHEMAS].
/// The types here are based on <https://github.com/DeterminateSystems/flake-schemas>
/// For example, see [NixosModule type](https://github.com/DeterminateSystems/flake-schemas/blob/0a5c42297d870156d9c57d8f99e476b738dcd982/flake.nix#L268)
#[allow(missing_docs)]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Type {
    #[serde(rename = "NixOS module")]
    NixosModule,
    #[serde(rename = "NixOS configuration")]
    NixosConfiguration,
    #[serde(rename = "nix-darwin configuration")]
    DarwinConfiguration,
    #[serde(rename = "package")]
    Package,
    #[serde(rename = "development environment")]
    DevShell,
    #[serde(rename = "CI test")]
    Check,
    #[serde(rename = "app")]
    App,
    #[serde(rename = "template")]
    Template,
    #[serde(other)]
    Unknown,
}

impl Type {
    /// Get the icon for this type
    pub fn to_icon(&self) -> &'static str {
        match self {
            Self::NixosModule => "❄️",
            Self::NixosConfiguration => "🔧",
            Self::DarwinConfiguration => "🍎",
            Self::Package => "📦",
            Self::DevShell => "🐚",
            Self::Check => "🧪",
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
