#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() {
    use clap::Parser;
    human_panic::setup_panic!();
    let args = nix_browser::cli::Args::parse();
    nix_browser::server::main(args).await
}

#[cfg(not(feature = "ssr"))]
fn main() {
    // No main entry point for wasm
}
