use leptos::*;

#[server(NixInfo, "/api")]
pub async fn nix_info() -> Result<String, ServerFnError> {
    use tokio::process::Command;
    let nix_version = Command::new("nix").arg("--version").output().await?.stdout;
    String::from_utf8(nix_version).map_err(|e| e.into())
}
