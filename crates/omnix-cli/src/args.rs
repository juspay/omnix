use clap::Parser;
use clap_verbosity_flag::{InfoLevel, Verbosity};

use crate::command::Command;

/// Omnix <https://omnix.page/>
//
// NOTE: Should we put this in `omnix` crate, and share with `omnix-gui` (see
// `omnix-gui/src/cli.rs`)?
#[derive(Parser, Debug)]
pub struct Args {
    #[command(flatten)]
    pub verbosity: Verbosity<InfoLevel>,

    #[clap(subcommand)]
    pub command: Command,
}
