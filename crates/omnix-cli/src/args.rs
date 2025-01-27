use clap::Parser;
use clap_verbosity_flag::{InfoLevel, Verbosity};

use crate::command::core::Command;

/// Omnix CLI entrypoint <https://omnix.page/>
#[derive(Parser, Debug)]
#[command(version, about)]
pub struct Args {
    #[command(flatten)]
    pub verbosity: Verbosity<InfoLevel>,

    #[clap(subcommand)]
    pub command: Command,
}
