#![feature(lazy_cell)]
use clap::Parser;
use clap_verbosity_flag::{InfoLevel, Verbosity};

mod command;

// NOTE: Should we put this in `omnix` crate, and share with `omnix-gui` (see
// `omnix-gui/src/cli.rs`)?
#[derive(Parser, Debug)]
pub struct Args {
    #[command(flatten)]
    pub verbosity: Verbosity<InfoLevel>,

    #[clap(subcommand)]
    pub command: command::Command,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    human_panic::setup_panic!();
    let args = Args::parse();
    let verbose = args.verbosity.log_level() > Some(clap_verbosity_flag::Level::Info);
    omnix::logging::setup_logging(&args.verbosity, !verbose);
    tracing::debug!("Args TEST @@@: {:?}", args);
    args.command.run(args.verbosity).await
}
