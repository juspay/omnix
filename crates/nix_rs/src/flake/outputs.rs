//! Nix flake outputs

use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, fmt::Display, path::Path};

use crate::system_list::SystemsListFlakeRef;

use super::{command::FlakeOptions, eval::nix_eval, url::FlakeUrl};

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

/// Represents the "outputs" of a flake
///
/// TODO: Rename this to `FlakeSchema` while generalizing the existing `schema.rs` module.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FlakeOutputs {
    pub inventory: BTreeMap<String, InventoryItem>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum InventoryItem {
    Leaf(Leaf),
    Attrset(BTreeMap<String, InventoryItem>),
}

impl FlakeOutputs {
    /// Determine flake outputs using [static@INSPECT_FLAKE] and [static@DEFAULT_FLAKE_SCHEMAS]
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

    pub fn get_inventory(&self, output: String) -> Option<&InventoryItem> {
        self.inventory.get(&output)
    }
}

impl InventoryItem {
    /// Get the non-attrset leaf
    pub fn as_leaf(&self) -> Option<&Leaf> {
        match self {
            Self::Leaf(v) => Some(v),
            _ => None,
        }
    }

    /// Ensure the value is an attrset, and get it
    pub fn as_attrset(&self) -> Option<&BTreeMap<String, InventoryItem>> {
        match self {
            Self::Attrset(v) => Some(v),
            _ => None,
        }
    }

    /// Lookup the given path in the output, returning a list of ([String], [Leaf]) pairs,
    /// where [String] represents the qualified attribute and [Leaf] its value.
    pub fn lookup_returning_qualified_attributes(
        &self,
        path: &[&str],
    ) -> Option<Vec<(String, Leaf)>> {
        if path.is_empty() {
            match self {
                Self::Attrset(_) => Some(self.to_qualified_attributes().into_iter().collect()),
                Self::Leaf(Leaf::Doc(_)) => Some(vec![]),
                Self::Leaf(v) => Some(vec![("".to_string(), v.clone())]),
            }
        } else {
            match self {
                Self::Attrset(v) => {
                    if let Some(children) = v.get("children") {
                        children.lookup_returning_qualified_attributes(path)
                    } else if let Some(output) = v.get("output") {
                        output.lookup_returning_qualified_attributes(path)
                    } else if let Some(entry) = v.get(path[0]) {
                        entry.lookup_returning_qualified_attributes(&path[1..])
                    } else {
                        None
                    }
                }
                _ => None,
            }
        }
    }

    /// Convert a [FlakeOutputs] to qualified attribute names and their corresponding leaf values.
    ///
    fn to_qualified_attributes(&self) -> Vec<(String, Leaf)> {
        match self {
            Self::Leaf(Leaf::Doc(_)) => vec![],
            Self::Leaf(v) => vec![(("".to_string()), v.clone())],
            Self::Attrset(v) => {
                // We want to skip "children" key in next recursive call
                // Also, if it is the last key before the leaf, we don't want to add "." in the end
                v.iter()
                    .flat_map(|(k, v)| {
                        v.to_qualified_attributes()
                            .into_iter()
                            .map(move |(k2, v2)| {
                                if k2.is_empty() {
                                    (k.to_string(), v2)
                                } else if k == "children" || k == "output" {
                                    (k2.to_string(), v2)
                                } else {
                                    (format!("{}.{}", k, &k2), v2)
                                }
                            })
                    })
                    .collect()
            }
        }
    }
}

/// Represents a leaf value of a flake output
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Leaf {
    Val(Val),
    Unknown(Unknown),
    Filtered(Filtered),
    Skipped(Skipped),
    /// Represents description for a flake output
    /// (e.g. `Doc` for `formatter` will be "The `formatter` output specifies the package to use to format the project.")
    Doc(String),
}

impl Leaf {
    /// Get the value as a [Val]
    pub fn as_val(&self) -> Option<&Val> {
        match self {
            Self::Val(v) => Some(v),
            _ => None,
        }
    }
}

/// The metadata of a flake output value which is of non-attrset [Type]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Val {
    #[serde(rename = "what")]
    pub type_: Type,
    /// The name derived from the derivation in the flake output
    pub derivation_name: Option<String>,
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

/// Boolean flags at the leaf of a flake output
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Unknown {
    pub unknown: bool,
}

/// Represents flake outputs that cannot be evaluated on current platform
/// (e.g. `nixosConfigurations` on darwin System)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename = "camelCase")]
pub struct Filtered {
    pub filtered: bool,
}

/// Represents flake outputs that are skipped unless explicitly requested
/// (e.g. `legacyPackages`)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Skipped {
    pub skipped: bool,
}

/// The type of a flake output [Val]
///
/// These types can differ based on [static@DEFAULT_FLAKE_SCHEMAS].
/// The types here are based on <https://github.com/DeterminateSystems/flake-schemas>
/// For example, see [NixosModule type](https://github.com/DeterminateSystems/flake-schemas/blob/0a5c42297d870156d9c57d8f99e476b738dcd982/flake.nix#L268)
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
