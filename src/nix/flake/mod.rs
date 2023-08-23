pub mod per_system;
pub mod show;
pub mod system;
pub mod url;

use leptos::*;
use serde::{Deserialize, Serialize};
use tracing::instrument;

use self::{per_system::PerSystemOutputs, show::FlakeShowOutput, system::System, url::FlakeUrl};

/// All the information about a Nix flake
// #[serde_as]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Flake {
    /// The flake url which this struct represents
    pub url: FlakeUrl,
    /// `nix flake show` output
    pub output: FlakeShowOutput,
    // TODO: Add higher-level info
    pub per_system: PerSystemOutputs,
}

/// Get [Flake] info for the given flake url
#[instrument(name = "flake")]
#[server(GetFlake, "/api")]
pub async fn get_flake(url: FlakeUrl) -> Result<Flake, ServerFnError> {
    use std::collections::BTreeMap;
    let output = self::show::run_nix_flake_show(&url).await?;
    let mut per_system = BTreeMap::new();
    for system in [
        "x86_64-linux",
        "x86_64-darwin",
        "aarch64-linux",
        "aarch64-darwin",
    ] {
        per_system.insert(
            System::from(system),
            per_system::SystemOutput::from(&output, &System::from(system)),
        );
    }
    Ok(Flake {
        url,
        output,
        per_system: PerSystemOutputs(per_system),
    })
}

impl IntoView for Flake {
    // TODO: Remove this attribute
    #[allow(clippy::iter_kv_map)]
    fn into_view(self, cx: Scope) -> View {
        view! { cx,
            <div class="flex flex-col my-4">
                <h3 class="text-lg font-bold">{self.url}</h3>

                <div class="font-mono text-sm">{self.output}</div>
            </div>
        }
        .into_view(cx)
    }
}
