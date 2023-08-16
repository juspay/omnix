//! Rust module for `nix flake show`

use leptos::*;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use tracing::instrument;

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
    #[serde(other)]
    Unknown,
}

#[instrument(name = "flake-show")]
#[server(GetNixFlakeShow, "/api")]
pub async fn get_nix_flake_show() -> Result<FlakeOutput, ServerFnError> {
    use tokio::process::Command;
    let mut cmd = Command::new("nix");
    cmd.args(vec![
        "--extra-experimental-features",
        "nix-command flakes",
        "flake",
        "show",
        "--allow-import-from-derivation",
        "--json",
        // TODO: Take arg from user?
        "github:nammayatri/nammayatri",
        // "github:srid/haskell-template",
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
        view! { cx, <span>{self.name} ", " {format!("{:?}", self.type_)} ", " {self.description}</span> }
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
