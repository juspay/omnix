//! Information about the user's Nix installation
use leptos::*;
use serde::{Deserialize, Serialize};
use tracing::instrument;

use crate::nix::config::NixConfig;

/// All the information about the user's Nix installation
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NixInfo {
    /// Nix version string
    pub nix_version: String,
    pub nix_config: NixConfig,
}

/// Determine [NixInfo] on the user's system
#[instrument(name = "nix-info")]
#[server(GetNixInfo, "/api")]
pub async fn get_nix_info() -> Result<NixInfo, ServerFnError> {
    use tokio::process::Command;
    let mut cmd = Command::new("nix");
    cmd.arg("--version");
    let stdout = crate::command::run_command(&mut cmd).await?;
    // TODO: Parse the version string
    let nix_version = String::from_utf8_lossy(&stdout).to_string();
    let nix_config = super::config::run_nix_show_config().await?;
    tracing::info!("Got nix info. Version = {}", nix_version);
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
                    <div class="p-1 my-1 rounded bg-primary-50">
                        <pre>{self.nix_version}</pre>
                    </div>
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
