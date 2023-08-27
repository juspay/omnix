//! Command-line interface
use clap::Parser;
use std::net::SocketAddr;
use tracing_subscriber::filter::{Directive, LevelFilter};

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
