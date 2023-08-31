//! Information about the user's Nix installation
use leptos::*;
use serde::{Deserialize, Serialize};

use crate::{config::NixConfig, version::NixVersion};

/// All the information about the user's Nix installation
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NixInfo {
    /// Nix version string
    pub nix_version: NixVersion,
    pub nix_config: NixConfig,
}

impl NixInfo {
    /// Determine [NixInfo] on the user's system
    #[cfg(feature = "ssr")]
    pub async fn from_nix(nix_cmd: &crate::command::NixCmd) -> Result<NixInfo, ServerFnError> {
        let nix_version = NixVersion::from_nix(nix_cmd).await?;
        let nix_config = NixConfig::from_nix(nix_cmd).await?;
        Ok(NixInfo {
            nix_version,
            nix_config,
        })
    }
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
