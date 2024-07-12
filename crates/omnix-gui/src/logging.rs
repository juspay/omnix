//! Logging setup for the server and client

use tracing_subscriber::filter::{Directive, LevelFilter};
use tracing_subscriber::EnvFilter;

pub fn setup_logging(verbosity: &Verbosity) {
    tracing_subscriber::fmt()
        .with_env_filter(verbosity.log_filter())
        .compact()
        .init();
}

#[derive(clap::Args, Debug, Clone)]
pub struct Verbosity {
    /// Server logging level
    ///
    /// Pass multiple v's (`-vvv...`) to increase logging level.
    #[arg(short = 'v', long = "verbose", action = clap::ArgAction::Count, default_value_t = 0)]
    pub verbose: u8,
}

impl Verbosity {
    /// Return the log filter for CLI flag.
    fn log_filter(&self) -> EnvFilter {
        self.log_directives()
            .iter()
            .fold(EnvFilter::from_env("OMNIX_LOG"), |filter, directive| {
                filter.add_directive(directive.clone())
            })
    }

    fn log_directives(&self) -> Vec<Directive> {
        // Allow warnings+errors from all crates.
        match self.verbose {
            // Default
            0 => vec![
                LevelFilter::WARN.into(),
                "omnix=info".parse().unwrap(),
                "nix_rs=info".parse().unwrap(),
                "nix_health=info".parse().unwrap(),
            ],
            // -v: log app DEBUG level, as well as http requests
            1 => vec![
                LevelFilter::WARN.into(),
                "omnix=debug".parse().unwrap(),
                "nix_rs=debug".parse().unwrap(),
                "nix_health=debug".parse().unwrap(),
            ],
            // -vv: log app TRACE level, as well as http requests
            2 => vec![
                LevelFilter::WARN.into(),
                "omnix=trace".parse().unwrap(),
                "nix_rs=trace".parse().unwrap(),
                "nix_health=trace".parse().unwrap(),
            ],
            // -vvv: log DEBUG level of app and libraries
            3 => vec![LevelFilter::DEBUG.into()],
            // -vvvv: log TRACE level of app and libraries
            _ => vec![LevelFilter::TRACE.into()],
        }
    }
}
