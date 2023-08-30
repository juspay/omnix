//! Information about the user's Nix installation
use leptos::*;
use serde::{Deserialize, Serialize};
use tracing::instrument;

use crate::{config::NixConfig, version::NixVersion};

/// All the information about the user's Nix installation
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NixInfo {
    /// Nix version string
    pub nix_version: NixVersion,
    pub nix_config: NixConfig,
}

/// Determine [NixInfo] on the user's system
#[instrument(name = "nix-info")]
#[server(GetNixInfo, "/api")]
pub async fn get_nix_info(_unit: ()) -> Result<NixInfo, ServerFnError> {
    let nix_version = super::version::run_nix_version().await?;
    let nix_config = super::config::run_nix_show_config().await?;
    Ok(NixInfo {
        nix_version,
        nix_config,
    })
}

impl IntoView for NixInfo {
    fn into_view(self, cx: Scope) -> View {
        view! { cx,
            <div class="flex flex-col p-4 space-y-8 bg-white border-2 rounded border-base-400">
                <div>
                    <b>
                        Nix Version
                    </b>
                    <div class="p-1 my-1 rounded bg-primary-50">{self.nix_version}</div>
                </div>
                <div>
                    <b>
                        Nix Config
                    </b>
                    {self.nix_config}
                </div>
            </div>
        }
        .into_view(cx)
    }
}
