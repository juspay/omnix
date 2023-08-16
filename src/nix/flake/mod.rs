use leptos::*;
use serde::Serialize;
use tracing::instrument;

use self::show::FlakeOutput;

pub mod show;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, serde::Deserialize)]
pub struct Flake {
    url: String,
    output: FlakeOutput,
}

#[instrument(name = "flake")]
#[server(GetNixFlakeShow, "/api")]
pub async fn get_flake() -> Result<Flake, ServerFnError> {
    let url = "github:nammayatri/nammayatri".to_string();
    let out = self::show::run_nix_flake_show(url.clone()).await?;
    Ok(Flake { url, output: out })
}

impl IntoView for Flake {
    fn into_view(self, cx: Scope) -> View {
        view! {cx,
        <div class="flex flex-col">
            <h3 class="font-bold">{self.url}</h3>
            <div>
            {self.output}
            </div>
        </div>
        }
        .into_view(cx)
    }
}
