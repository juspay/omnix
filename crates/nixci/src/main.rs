use anyhow::Result;
use nixci::cli;

#[tokio::main]
async fn main() -> Result<()> {
    let args = cli::CliArgs::parse().await?;
    nixci::logging::setup_logging(args.verbose);
    nixci::nixci(args).await?;
    Ok(())
}
