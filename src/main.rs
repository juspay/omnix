#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() {
    nix_browser::server::main().await
}

#[cfg(not(feature = "ssr"))]
fn main() {
    // No main entry point for wasm
}
