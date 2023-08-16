use leptos::*;
use serde::Serialize;
use tracing::instrument;

use self::show::FlakeOutput;

pub mod show;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, serde::Deserialize)]
pub struct Flake {
    output: FlakeOutput,
}

#[instrument(name = "flake")]
#[server(GetNixFlakeShow, "/api")]
pub async fn get_flake() -> Result<Flake, ServerFnError> {
    let out = self::show::run_nix_flake_show().await?;
    Ok(Flake { output: out })
}

impl IntoView for Flake {
    fn into_view(self, cx: Scope) -> View {
        self.output.into_view(cx)
    }
}
