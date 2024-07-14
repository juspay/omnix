//! Command-line interface
use clap::Parser;

#[derive(Parser, Debug)]
pub struct Args {
    #[command(flatten)]
    pub verbosity: omnix::logging::Verbosity,
}
