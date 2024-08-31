use clap::Parser;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    human_panic::setup_panic!();
    let args = omnix_cli::args::Args::parse();
    let verbose = args.verbosity.log_level() > Some(clap_verbosity_flag::Level::Info);
    omnix_common::logging::setup_logging(&args.verbosity, !verbose);
    tracing::debug!("Args: {:?}", args);
    args.command.run(args.verbosity).await
}
