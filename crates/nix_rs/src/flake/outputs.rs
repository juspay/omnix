//! Nix flake outputs

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
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
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub enum FlakeOutputs {
    Doc(String),
    Unknown(Unknown),
    Filtered(Filtered),
    Skipped(Skipped),
    Val(Val),
    Attrset(BTreeMap<String, FlakeOutputs>),
}

impl Serialize for FlakeOutputs {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        use serde::ser::SerializeMap;

        match self {
            Self::Doc(_) => serializer.serialize_none(),
            Self::Unknown(v) => v.serialize(serializer),
            Self::Filtered(v) => v.serialize(serializer),
            Self::Skipped(v) => v.serialize(serializer),
            Self::Val(v) => v.value.serialize(serializer),
            Self::Attrset(v) => {
                let mut map = serializer.serialize_map(Some(v.len()))?;
                for (k, v) in v {
                    if let FlakeOutputs::Doc(_) = v {
                        // Skip Doc variant
                        // TODO: This can be avoided if [FlakeOutputs] identifies `output`
                        // key of a given flake output and only serialize the value within it.
                        continue;
                    }
                    map.serialize_entry(k, v)?;
                }
                map.end()
            }
        }
    }
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
    pub fn as_val(&self) -> Option<&Val> {
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

    /// Flatten [FlakeOutputs] returning [Value] by extending the values `children` and `output` keys
    /// to the parent key.
    ///
    /// A `json` that is of the form:
    /// ```json
    /// {
    ///    attr1: {
    ///       children: {
    ///         attr2: {
    ///           leaf: {
    ///             what: "omnix config",
    ///             value: true,
    ///          },
    ///        },
    ///     },
    ///   },
    /// }
    /// ```
    /// will be flattened to:
    /// ```json
    /// {
    ///   attr1: {
    ///    attr2: true
    ///  }
    /// }
    /// ```
    pub fn as_flattened_value(&self) -> Result<Value, serde_json::Error> {
        serde_json::to_value(self).map(Self::flatten_children_and_output)
    }

    fn flatten_children_and_output(value: Value) -> Value {
        match value {
            Value::Object(map) => {
                let new_map = map.into_iter().fold(Map::new(), |mut acc, (key, value)| {
                    let new_value = Self::flatten_children_and_output(value);
                    match key.as_str() {
                        "children" | "output" => {
                            if let Value::Object(inner_map) = new_value {
                                acc.extend(inner_map);
                            } else {
                                acc.insert(key, new_value);
                            }
                        }
                        _ => {
                            acc.insert(key, new_value);
                        }
                    }
                    acc
                });
                Value::Object(new_map)
            }
            Value::Array(vec) => Value::Array(
                vec.into_iter()
                    .map(Self::flatten_children_and_output)
                    .collect(),
            ),
            _ => value,
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
    #[serde(rename = "what")]
    pub type_: Type,
    pub derivation_name: Option<String>,
    pub short_description: Option<String>,
    pub value: Option<serde_json::Value>,
}

impl Default for Val {
    fn default() -> Self {
        Self {
            type_: Type::Unknown,
            derivation_name: None,
            short_description: None,
            value: None,
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
    Doc(String),
    Unknown(Unknown),
    Filtered(Filtered),
    Skipped(Skipped),
    Val(Val),
    Attrset(BTreeMap<String, FlakeOutputsUntagged>),
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
            Self::Doc(v) => FlakeOutputs::Doc(v),
            Self::Unknown(v) => FlakeOutputs::Unknown(v),
            Self::Filtered(v) => FlakeOutputs::Filtered(v),
            Self::Skipped(v) => FlakeOutputs::Skipped(v),
            Self::Val(v) => FlakeOutputs::Val(v),
            Self::Attrset(v) => FlakeOutputs::Attrset(
                v.into_iter()
                    .map(|(k, v)| (k, v.into_flake_outputs()))
                    .collect(),
            ),
        }
    }
}
