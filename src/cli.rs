//! Command-line interface
use clap::Parser;

#[derive(Parser, Debug)]
pub struct Args {
    /// Do not automatically open the application in the local browser
    ///
    /// Enabled by default if the app is running under `cargo leptos ...`
    #[arg(short = 'n', long = "no-open", default_value_t = in_cargo_leptos())]
    pub no_open: bool,
    ///This flag enables the TRACE and DEBUG level log.
    #[arg(long = "vv", default_value_t = in_cargo_leptos())]
    pub vv: bool,
    ///This flag enables the DEBUG level log.
    #[arg(short = 'v', long = "verbose", default_value_t = in_cargo_leptos())]
    pub verbose: bool,
}

/// Whether the app is running under `cargo leptos ...`
fn in_cargo_leptos() -> bool {
    std::env::var("LEPTOS_OUTPUT_NAME").is_ok()
}
