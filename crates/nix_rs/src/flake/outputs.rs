//! Nix flake outputs

use serde::{Deserialize, Serialize};
use std::{
    collections::{btree_map::Entry, BTreeMap},
    fmt::Display,
};

use super::url::FlakeUrl;

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
    Doc(String),
    Unknown(Unknown),
    Filtered(Filtered),
    Skipped(Skipped),
    Val(Val),
    Attrset(BTreeMap<String, FlakeOutputs>),
}

/// A filtered version of [FlakeOutputs]
///
/// This is used to filter out the `children` and `output` keys from the flake outputs.
/// Along with filtering out the unnecessary metadata keys from the flake outputs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum FilteredFlakeOutputs {
    #[serde(serialize_with = "value_serializer")]
    Val(Val),
    Attrset(BTreeMap<String, FilteredFlakeOutputs>),
}

fn value_serializer<S>(val: &Val, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    val.value.serialize(serializer)
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

    /// Convert to [FilteredFlakeOutputs]
    pub fn into_filtered_flake_outputs(self) -> FilteredFlakeOutputs {
        match self {
            Self::Val(v) => FilteredFlakeOutputs::Val(v),
            Self::Attrset(map) => {
                let filtered_map: BTreeMap<String, FilteredFlakeOutputs> = map
                    .into_iter()
                    .fold(BTreeMap::new(), |mut acc, (key, value)| {
                        let filtered_v = value.into_filtered_flake_outputs();
                        match key.as_str() {
                            "children" | "output" => {
                                if let FilteredFlakeOutputs::Attrset(inner_map) = filtered_v {
                                    acc.extend(inner_map);
                                } else {
                                    acc.insert(key, filtered_v);
                                }
                            }
                            _ => {
                                if !matches!(filtered_v, FilteredFlakeOutputs::Attrset(ref m) if m.is_empty()) {
                                    acc.insert(key, filtered_v);
                                }
                            }
                        }
                        acc
                    });

                FilteredFlakeOutputs::Attrset(filtered_map)
            }
            _ => FilteredFlakeOutputs::Attrset(BTreeMap::new()),
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

impl FilteredFlakeOutputs {
    /// Run `nix flake show` on the given flake url and filter the outputs
    pub async fn from_nix(
        nix_cmd: &crate::command::NixCmd,
        flake_url: &super::url::FlakeUrl,
    ) -> Result<Self, crate::command::NixCmdError> {
        let v = FlakeOutputs::from_nix(nix_cmd, flake_url).await?;
        Ok(v.into_filtered_flake_outputs())
    }

    /// Deserialize the value into the given type
    pub fn deserialize_into<T>(&self) -> T
    where
        T: Default + serde::de::DeserializeOwned,
    {
        serde_json::to_value(self)
            .ok()
            .and_then(|v| serde_json::from_value(v).ok())
            .unwrap_or_default()
    }

    /// Find the qualified attribute in the flake outputs
    pub async fn find_qualified_attr<T, S>(
        &self,
        url: &FlakeUrl,
        root_attrs: &[S],
    ) -> Result<(T, Vec<String>), QualifiedAttrError>
    where
        S: AsRef<str>,
        T: Default + serde::de::DeserializeOwned + std::fmt::Debug,
    {
        for root_attr in root_attrs {
            if let Some(v) = self.find_nested_output(root_attr.as_ref()) {
                return Ok((v, url.get_attr().as_list()));
            }
        }
        match url.get_attr().0 {
            None => Ok((Default::default(), vec![])),
            Some(attr) => Err(QualifiedAttrError::UnexpectedAttribute(attr)),
        }
    }

    /// Find the nested output in the flake outputs and deserialize it into the given type
    pub fn find_nested_output<T>(&self, root_attr: &str) -> Option<T>
    where
        T: Default + serde::de::DeserializeOwned,
    {
        root_attr
            .split(".")
            .try_fold(self, |acc, key| acc.get(key))
            .map(|result| result.deserialize_into())
    }

    /// Get the value of the given key in the attrset
    pub fn get(&self, key: &str) -> Option<&FilteredFlakeOutputs> {
        match self {
            FilteredFlakeOutputs::Val(_) => None,
            FilteredFlakeOutputs::Attrset(map) => map.get(key),
        }
    }
}
#[derive(Debug, thiserror::Error)]
pub enum QualifiedAttrError {
    /// The attribute was not found in the flake outputs
    #[error("Unexpected attribute, when config not present in flake: {0}")]
    UnexpectedAttribute(String),
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
