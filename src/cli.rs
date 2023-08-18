//! Command-line interface
use clap::Parser;

#[derive(Parser, Debug)]
pub struct Args {
    /// Do not automatically open the application in the local browser
    ///
    /// Enabled by default if the app is running under `cargo leptos ...`
    #[arg(short = 'n', long = "no-open", default_value_t = in_cargo_leptos())]
    pub no_open: bool,
    /// This flag alters the level log based on count of arg.
    ///
    /// 1 -> DEBUG, 2 -> TRACE, Default -> INFO
    #[arg(short = 'v', long = "verbose", action = clap::ArgAction::Count , default_value_t = 0)]
    pub verbose: u8,
}

impl Args {
    pub fn log_level(&self) -> tracing::Level {
        match self.verbose {
            0 => tracing::Level::INFO,
            1 => tracing::Level::DEBUG,
            _ => tracing::Level::TRACE,
        }
    }
}

/// Whether the app is running under `cargo leptos ...`
fn in_cargo_leptos() -> bool {
    std::env::var("LEPTOS_OUTPUT_NAME").is_ok()
}
