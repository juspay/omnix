//! Nix flake outputs

use leptos::*;
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
    /// Example:
    /// let val = tree.pop(&["packages", "aarch64-darwin", "default"]);
    pub fn pop(&mut self, path: &[&str]) -> Option<Self> {
        let mut cur = self;
        let mut path = path.iter().peekable();
        while let Some(part) = path.next() {
            match cur {
                Self::Attrset(v) => {
                    if let Entry::Occupied(entry) = v.entry(part.to_string()) {
                        if path.peek().is_none() {
                            return Some(entry.remove());
                        } else {
                            cur = entry.into_mut();
                        }
                    } else {
                        return None;
                    }
                }
                _ => return None,
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

/// The [IntoView] instance for [FlakeOutputs] renders it recursively. This view
/// is used to see the raw flake output only; it is not useful for general UX.
///
/// WARNING: This may cause performance problems if the tree is large.
impl IntoView for FlakeOutputs {
    fn into_view(self, cx: Scope) -> View {
        match self {
            Self::Val(v) => v.into_view(cx),
            Self::Attrset(v) => view! { cx,
                <ul class="list-disc">
                    {v
                        .iter()
                        .map(|(k, v)| {
                            view! { cx,
                                <li class="ml-4">
                                    <span class="px-2 py-1 font-bold text-primary-500">{k}</span>
                                    {v.clone()}
                                </li>
                            }
                        })
                        .collect_view(cx)}
                </ul>
            }
            .into_view(cx),
        }
    }
}

impl IntoView for Val {
    fn into_view(self, cx: Scope) -> View {
        view! { cx,
            <span>
                <b>{self.name}</b>
                " ("
                {self.type_}
                ") "
                <em>{self.description}</em>
            </span>
        }
        .into_view(cx)
    }
}

impl IntoView for Type {
    fn into_view(self, cx: Scope) -> View {
        view! { cx,
            <span>
                {match self {
                    Self::NixosModule => "nixosModule ❄️",
                    Self::Derivation => "derivation 📦",
                    Self::App => "app 📱",
                    Self::Template => "template 🏗️",
                    Self::Unknown => "unknown ❓",
                }}

            </span>
        }
        .into_view(cx)
    }
}
