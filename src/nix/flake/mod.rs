//! Rust module for Nix flakes
pub mod outputs;
pub mod schema;
#[cfg(feature = "ssr")]
pub mod show;
pub mod system;
pub mod url;

use leptos::*;
use leptos_router::*;
use serde::{Deserialize, Serialize};
use tracing::instrument;

use self::{outputs::FlakeOutputs, schema::FlakeSchema, system::System, url::FlakeUrl};

/// All the information about a Nix flake
// #[serde_as]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Flake {
    /// The flake url which this struct represents
    pub url: FlakeUrl,
    /// `nix flake show` output
    pub output: FlakeOutputs,
    /// Flake output schema (typed version of [FlakeOutputs])
    pub schema: FlakeSchema,
    // TODO: Add `nix flake metadata` info.
}

/// Get [Flake] info for the given flake url
#[instrument(name = "flake")]
#[server(GetFlake, "/api")]
pub async fn get_flake(url: FlakeUrl) -> Result<Flake, ServerFnError> {
    use super::config::run_nix_show_config;
    // TODO: Can we cache this?
    let nix_config = run_nix_show_config().await?;
    let system = nix_config.system.value;
    let output = self::show::run_nix_flake_show(&url).await?;
    Ok(Flake {
        url,
        output: output.clone(),
        schema: FlakeSchema::from(&output, &system),
    })
}

impl IntoView for Flake {
    fn into_view(self, cx: Scope) -> View {
        view! { cx,
            <div class="flex flex-col my-4">
                <h3 class="text-lg font-bold">{self.url}</h3>
                <div class="text-sm italic text-gray-600">
                    <A href="/flake/raw" exact=true>
                        "View raw output"
                    </A>
                </div>
                <div>{self.schema}</div>
            </div>
        }
        .into_view(cx)
    }
}
