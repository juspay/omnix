pub mod show;
pub mod url;

use std::fmt::Formatter;
use std::str::FromStr;
use std::{fmt::Display, hash::Hash};

use leptos::*;
use serde::{Deserialize, Serialize};
use tracing::instrument;

use self::{show::FlakeOutput, url::FlakeUrl};

/// All the information about a Nix flake
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Flake {
    /// The flake url which this struct represents
    url: FlakeUrl,
    /// `nix flake show` output
    output: FlakeOutput,
}

#[instrument(name = "flake")]
#[server(GetNixFlake, "/api")]
pub async fn get_flake(url: FlakeUrl) -> Result<Flake, ServerFnError> {
    // TODO Let the user enter this from UI (input box)
    // let url = "github:nammayatri/nammayatri".to_string();
    let out = self::show::run_nix_flake_show(&url).await?;
    Ok(Flake { url, output: out })
}
// TODO: automate these instances
impl FromStr for GetNixFlake {
    type Err = ServerFnError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let v = FlakeUrl::from_str(s).map_err(|e| ServerFnError::ServerError(e.to_string()))?;
        Ok(GetNixFlake { url: v })
    }
}
impl Hash for GetNixFlake {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.url.hash(state)
    }
}
impl PartialEq for GetNixFlake {
    fn eq(&self, other: &Self) -> bool {
        self.url == other.url
    }
}
impl Eq for GetNixFlake {}
impl Display for GetNixFlake {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.url)
    }
}

impl IntoView for Flake {
    fn into_view(self, cx: Scope) -> View {
        view! { cx,
            <div class="flex flex-col my-4">
                <h3 class="text-lg font-bold">{self.url}</h3>
                <div>{self.output}</div>
            </div>
        }
        .into_view(cx)
    }
}
