pub mod show;
pub mod url;

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
    // TODO: Add higher-level info
}

/// Get [Flake] info for the given flake url
#[instrument(name = "flake")]
#[server(GetFlake, "/api")]
pub async fn get_flake(url: FlakeUrl) -> Result<Flake, ServerFnError> {
    let out = self::show::run_nix_flake_show(&url).await?;
    Ok(Flake { url, output: out })
}

impl IntoView for Flake {
    fn into_view(self, cx: Scope) -> View {
        view! { cx,
            <div class="flex flex-col my-4">
                <h3 class="text-lg font-bold">{self.url}</h3>
                <p class="my-2">
                    TODO: Show overview, rather than raw flake output
                </p>
                <div class="font-mono text-sm">{self.output}</div>
            </div>
        }
        .into_view(cx)
    }
}
