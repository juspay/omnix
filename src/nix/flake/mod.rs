use leptos::*;
use serde::Serialize;
use tracing::instrument;

use self::show::FlakeOutput;

pub mod show;

/// All the information about a Nix flake
#[derive(Debug, Clone, PartialEq, Eq, Serialize, serde::Deserialize)]
pub struct Flake {
    /// The flake url which this struct represents
    url: String,
    /// `nix flake show` output
    output: FlakeOutput,
}

#[instrument(name = "flake")]
#[server(GetNixFlake, "/api")]
pub async fn get_flake(url: String) -> Result<Flake, ServerFnError> {
    // TODO Let the user enter this from UI (input box)
    // let url = "github:nammayatri/nammayatri".to_string();
    let out = self::show::run_nix_flake_show(url.clone()).await?;
    Ok(Flake { url, output: out })
}

impl IntoView for Flake {
    fn into_view(self, cx: Scope) -> View {
        view! { cx,
            <div class="flex flex-col">
                <h3 class="font-bold">{self.url}</h3>
                <div>{self.output}</div>
            </div>
        }
        .into_view(cx)
    }
}
