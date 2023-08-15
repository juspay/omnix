mod cli;

#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() {
    human_panic::setup_panic!();
    let args = cli::parse();
    nix_browser::server::main(args.no_open).await
}

#[cfg(not(feature = "ssr"))]
fn main() {
    // No main entry point for wasm
}
