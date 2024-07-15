use clap::Parser;

mod command;

// NOTE: Should we put this in `omnix` crate, and share with `omnix-gui` (see
// `omnix-gui/src/cli.rs`)?
#[derive(Parser, Debug)]
pub struct Args {
    #[command(flatten)]
    pub verbosity: omnix::logging::Verbosity,

    #[clap(subcommand)]
    pub command: command::Command,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    omnix::logging::setup_logging(&args.verbosity);
    tracing::debug!("Args: {:?}", args);
    args.command.run().await
}
