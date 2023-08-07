use leptos::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
/// Information about the user's Nix installation
pub struct NixInfo {
    /// Nix version string
    pub nix_version: String,
}

#[server(GetNixInfo, "/api")]
pub async fn get_nix_info() -> Result<NixInfo, ServerFnError> {
    use tokio::process::Command;
    let out = Command::new("nix").arg("--version").output().await?.stdout;
    // TODO: Parse the version string
    let nix_version = String::from_utf8(out)
        .map_err(|e| <std::string::FromUtf8Error as Into<ServerFnError>>::into(e))?;
    Ok(NixInfo { nix_version })
}

impl IntoView for NixInfo {
    fn into_view(self, cx: Scope) -> View {
        view! {cx,
            <div class="p-2 bg-blue-100 border-2 border-black rounded shadow-md">
                <b>Nix Version:</b>
                <pre>{self.nix_version}</pre>
            </div>
        }
        .into_view(cx)
    }
}
