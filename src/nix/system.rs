//! Information about the user's System
use leptos::*;
use serde::{Deserialize, Serialize};
use std::env;
use tracing::instrument;

/// All the information about the user's Nix installation
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SysInfo {
    /// value of $USER
    pub current_user: String,
}

/// Determine [SysInfo] on the user's system
#[instrument(name = "sys-info")]
#[server(GetNixInfo, "/api")]
pub async fn get_sys_info(_unit: ()) -> Result<SysInfo, ServerFnError> {
    Ok(SysInfo {
        current_user: env::var("USER")?,
    })
}

impl IntoView for SysInfo {
    fn into_view(self, cx: Scope) -> View {
        view! { cx,
            <div class="flex flex-col p-4 space-y-8 bg-white border-2 rounded border-base-400">
                <div>
                    <b>
                        Current User
                    </b>
                    <div class="p-1 my-1 rounded bg-primary-50">{self.current_user}</div>
                </div>
            </div>
        }
        .into_view(cx)
    }
}
