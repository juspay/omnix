//! Command-line interface
use clap::Parser;
use clap_verbosity_flag::{InfoLevel, Verbosity};

#[derive(Parser, Debug)]
pub struct Args {
    #[command(flatten)]
    pub verbosity: Verbosity<InfoLevel>,
}
