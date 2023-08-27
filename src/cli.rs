//! Command-line interface
use clap::Parser;
use tracing_subscriber::filter::{Directive, LevelFilter};
#[derive(Parser, Debug)]
pub struct Args {
    /// Do not automatically open the application in the local browser
    ///
    /// Enabled by default if the app is running under `cargo leptos ...`
    #[arg(short = 'n', long = "no-open", default_value_t = in_cargo_leptos())]
    pub no_open: bool,
    /// Be verbose in server logging (-v, -vv)
    #[arg(short = 'v', long = "verbose", action = clap::ArgAction::Count , default_value_t = 0)]
    pub verbose: u8,
}

impl Args {
    /// Return the server log level
    pub fn log_level(&self) -> tracing::Level {
        match self.verbose {
            0 => tracing::Level::INFO,
            1 => tracing::Level::DEBUG,
            _ => tracing::Level::TRACE,
        }
    }
    /// Return the server log directive
    pub fn log_directives(&self) -> Vec<Directive> {
        match self.verbose {
            0 => vec![
                LevelFilter::INFO.into(),
                "nix_browser=info".parse().unwrap(),
                "tower_http=OFF".parse().unwrap(),
                "hyper=OFF".parse().unwrap(),
            ],
            1 => vec![
                LevelFilter::DEBUG.into(),
                "nix_browser=debug".parse().unwrap(),
                "tower_http=OFF".parse().unwrap(),
                "hyper=OFF".parse().unwrap(),
            ],
            2 => vec![
                LevelFilter::TRACE.into(),
                "nix_browser=trace".parse().unwrap(),
                "tower_http=OFF".parse().unwrap(),
                "hyper=OFF".parse().unwrap(),
            ],
            3 => vec![LevelFilter::DEBUG.into()],
            _ => vec![LevelFilter::TRACE.into()],
        }
    }
}

/// Whether the app is running under `cargo leptos ...`
fn in_cargo_leptos() -> bool {
    std::env::var("LEPTOS_OUTPUT_NAME").is_ok()
}
