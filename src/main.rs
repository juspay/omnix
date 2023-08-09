#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() {
    human_panic::setup_panic!();
    nix_browser::server::main().await
}

#[cfg(not(feature = "ssr"))]
fn main() {
    // No main entry point for wasm
}
