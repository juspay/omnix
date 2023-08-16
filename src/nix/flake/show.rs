//! Rust module for `nix flake show`

#[cfg(feature = "ssr")]
use super::url::FlakeUrl;
use leptos::*;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum FlakeOutput {
    Leaf(Leaf),
    Attrset(FlakeOutputSet),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FlakeOutputSet(BTreeMap<String, FlakeOutput>);

impl FlakeOutput {
    pub fn as_leaf(&self) -> Option<&Leaf> {
        match self {
            Self::Leaf(v) => Some(v),
            _ => None,
        }
    }

    pub fn as_attrset(&self) -> Option<&BTreeMap<String, FlakeOutput>> {
        match self {
            Self::Attrset(v) => Some(&v.0),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Leaf {
    #[serde(rename = "type")]
    pub type_: Type,
    pub name: Option<String>,
    pub description: Option<String>,
}

// https://github.com/NixOS/nix/blob/2.14.1/src/nix/flake.cc#L1105
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

#[cfg(feature = "ssr")]
pub async fn run_nix_flake_show(flake_url: &FlakeUrl) -> Result<FlakeOutput, ServerFnError> {
    use tokio::process::Command;

    let mut cmd = Command::new("nix");
    cmd.args(vec![
        "--extra-experimental-features",
        "nix-command flakes",
        "flake",
        "show",
        "--allow-import-from-derivation",
        "--json",
        &flake_url.to_string(),
    ]);
    let stdout: Vec<u8> = crate::command::run_command(&mut cmd).await?;
    let v = serde_json::from_slice::<FlakeOutput>(&stdout)?;
    Ok(v)
}

impl IntoView for FlakeOutput {
    fn into_view(self, cx: Scope) -> View {
        match self {
            Self::Leaf(v) => v.into_view(cx),
            Self::Attrset(v) => v.into_view(cx),
        }
    }
}

impl IntoView for Leaf {
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

impl IntoView for FlakeOutputSet {
    fn into_view(self, cx: Scope) -> View {
        view! { cx,
            <ul class="list-disc">
                {self
                    .0
                    .iter()
                    .map(|(k, v)| {
                        view! { cx, <li class="ml-2">{k} : {v.clone()}</li> }
                    })
                    .collect_view(cx)}
            </ul>
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

#[cfg(feature = "ssr")]
#[tokio::test]
#[ignore] // Requires network, so won't work in Nix
async fn test_nix_flake_show() {
    // Test on a flake with IFD
    let flake_url = "github:srid/haskell-template".into();
    assert!(run_nix_flake_show(&flake_url).await.is_ok());
}
