//! Nix flake outputs

use serde::{Deserialize, Serialize};
use std::{
    collections::{btree_map::Entry, BTreeMap},
    fmt::Display,
};

/// Absolute path to the `nix` binary compiled with flake schemas support
///
/// We expect this environment to be set in Nix build and shell.
pub const NIX_FLAKE_SCHEMAS_BIN: &str = env!("NIX_FLAKE_SCHEMAS_BIN");

/// Flake URL of the default flake schemas
///
/// We expect this environment to be set in Nix build and shell.
pub const DEFAULT_FLAKE_SCHEMAS: &str = env!("DEFAULT_FLAKE_SCHEMAS");

/// Represents the "outputs" of a flake
///
/// This structure is currently produced by `nix flake show`, thus to parse it we must toggle serde untagged.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FlakeOutputs {
    Leaf(Leaf),
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

    /// Get the non-attrset leaf
    pub fn as_leaf(&self) -> Option<&Leaf> {
        match self {
            Self::Leaf(v) => Some(v),
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

    /// Lookup the given output prefix, returning a list of ([String], [Leaf]) pairs.
    ///
    /// Given the following json output of `nix flake show`:
    /// ```json
    /// {
    /// "packages": {
    ///   "doc": "The `packages` flake output contains packages that can be added to a shell using `nix shell`.\n",
    ///   "output": {
    ///     "children": {
    ///       "aarch64-darwin": {
    ///         "children": {
    ///           "cargo-doc-live": {
    ///             "derivationName": "cargo-doc-live",
    ///             "leaf": true,
    ///             "what": "package"
    ///           },
    ///           "default": {
    ///             "derivationName": "omnix-cli-0.1.0",
    ///             "leaf": true,
    ///             "shortDescription": "Improve developer experience of using Nix",
    ///             "what": "package"
    ///           },
    ///           "gui": {
    ///             "derivationName": "omnix-gui-0.1.0",
    ///             "leaf": true,
    ///             "shortDescription": "Graphical interface for Omnix",
    ///             "what": "package"
    ///           }
    ///         }
    ///       }
    ///     }
    ///   }
    /// }
    /// ```
    ///
    /// And given the prefix `packages` we get:
    /// ```rust,ignore
    /// Some([
    ///   ("doc", Doc("The `packages` flake output contains packages that can be added to a shell using `nix shell`.\n")),
    ///   ("aarch64-darwin.cargo-doc-live", Val(Val { type_: Package, derivation_name: Some("cargo-doc-live"), short_description: None, value: None })),
    ///   ("aarch64-darwin.default", Val(Val { type_: Package, derivation_name: Some("omnix-cli-0.1.0"), short_description: Some("Improve developer experience of using Nix"), value: None })),
    ///   ("aarch64-darwin.gui", Val(Val { type_: Package, derivation_name: Some("omnix-gui-0.1.0"), short_description: Some("Graphical interface for Omnix"), value: None })),
    /// ])
    /// ```
    /// And similarly, for the prefix `packages.aarch64-darwin` we get:
    /// ```rust,ignore
    /// Some([
    ///   ("cargo-doc-live", Val(Val { type_: Package, derivation_name: Some("cargo-doc-live"), short_description: None, value: None })),
    ///   ("default", Val(Val { type_: Package, derivation_name: Some("omnix-cli-0.1.0"), short_description: Some("Improve developer experience of using Nix"), value: None })),
    ///   ("gui", Val(Val { type_: Package, derivation_name: Some("omnix-gui-0.1.0"), short_description: Some("Graphical interface for Omnix"), value: None })),
    /// ])
    /// ```
    pub fn lookup_returning_qualified_attributes(
        &self,
        prefix: &[&str],
    ) -> Option<Vec<(String, Leaf)>> {
        if prefix.is_empty() {
            match self {
                Self::Attrset(_) => Some(self.to_qualified_attributes().into_iter().collect()),
                Self::Leaf(Leaf::Doc(_)) => Some(vec![]),
                Self::Leaf(v) => Some(vec![("".to_string(), v.clone())]),
            }
        } else {
            match self {
                Self::Attrset(v) => {
                    if let Some(children) = v.get("children") {
                        children.lookup_returning_qualified_attributes(prefix)
                    } else if let Some(output) = v.get("output") {
                        output.lookup_returning_qualified_attributes(prefix)
                    } else if let Some(entry) = v.get(prefix[0]) {
                        entry.lookup_returning_qualified_attributes(&prefix[1..])
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
/// These types can differ based on [DEFAULT_FLAKE_SCHEMAS].
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

/// This type is identical to [FlakeOutputs] except for the serde untagged attribute, which enables parsing the JSON output of `nix flake show`.
///
/// This separation exists to workaround <https://github.com/DioxusLabs/dioxus-std/issues/20>
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
enum FlakeOutputsUntagged {
    ULeaf(Leaf),
    UAttrset(BTreeMap<String, FlakeOutputsUntagged>),
}

impl FlakeOutputsUntagged {
    /// Run `nix flake show` on the given flake url
    #[tracing::instrument(name = "flake-show")]
    async fn from_nix(
        nix_cmd: &crate::command::NixCmd,
        flake_url: &super::url::FlakeUrl,
    ) -> Result<Self, crate::command::NixCmdError> {
        let mut nix_flake_schemas_cmd = nix_cmd.clone();
        nix_flake_schemas_cmd.command = Some(env!("NIX_FLAKE_SCHEMAS_BIN").to_string());

        let v = nix_flake_schemas_cmd
            .run_with_args_expecting_json(&[
                "flake",
                "show",
                "--legacy", // for showing nixpkgs legacyPackages
                "--allow-import-from-derivation",
                "--json",
                "--default-flake-schemas",
                env!("DEFAULT_FLAKE_SCHEMAS"),
                flake_url,
            ])
            .await?;
        Ok(v)
    }

    /// Convert to [FlakeOutputs]
    fn into_flake_outputs(self) -> FlakeOutputs {
        match self {
            Self::ULeaf(v) => FlakeOutputs::Leaf(v),
            Self::UAttrset(v) => FlakeOutputs::Attrset(
                v.into_iter()
                    .map(|(k, v)| (k, v.into_flake_outputs()))
                    .collect(),
            ),
        }
    }
}
