//! Command-line interface
use clap::Parser;

#[derive(Parser, Debug)]
pub struct Args {
    /// Do not automatically open the application in the local browser
    ///
    /// Enabled by default if the app is running under `cargo leptos ...`
    #[arg(short = 'n', long = "no-open", default_value_t = in_cargo_leptos())]
    pub no_open: bool,

    /// Provides a way to control the address leptos is served from
    #[arg(short = 's', long = "site-addr")]
    pub leptos_site_addr: Option<String>,
}

/// Whether the app is running under `cargo leptos ...`
fn in_cargo_leptos() -> bool {
    std::env::var("LEPTOS_OUTPUT_NAME").is_ok()
}
