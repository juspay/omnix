//! Command-line interface
use clap::Parser;

use crate::logging;

#[derive(Parser, Debug)]
pub struct Args {
    #[command(flatten)]
    pub verbosity: logging::Verbosity,
}
