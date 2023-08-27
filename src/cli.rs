//! Command-line interface
use clap::Parser;
use std::net::SocketAddr;
use tracing_subscriber::EnvFilter;

use crate::logging;

#[derive(Parser, Debug)]
pub struct Args {
    /// Do not automatically open the application in the local browser
    ///
    /// Enabled by default if the app is running under `cargo leptos ...`
    #[arg(short = 'n', long = "no-open", env = "NIX_BROWSER_NO_OPEN")]
    pub no_open: bool,

    /// The address to serve the application on
    ///
    /// Format: `IP_ADDRESS:PORT`
    ///
    /// Uses localhost and random port by default. To use a different port, pass
    /// `127.0.0.1:8080`
    #[arg(
        short = 's',
        long = "site-addr",
        default_value = "127.0.0.1:0",
        env = "LEPTOS_SITE_ADDR"
    )]
    pub site_addr: Option<SocketAddr>,

    /// Be verbose in server logging (-v, -vv)
    #[arg(short = 'v', long = "verbose", action = clap::ArgAction::Count , default_value_t = 0)]
    pub verbose: u8,
}

impl Args {
    /// Return the log filter for CLI flag.
    pub fn log_filter(&self) -> EnvFilter {
        logging::log_directives_for_verbosity(self.verbose)
            .iter()
            .fold(
                EnvFilter::from_env("NIX_BROWSER_LOG"),
                |filter, directive| filter.add_directive(directive.clone()),
            )
    }
}
