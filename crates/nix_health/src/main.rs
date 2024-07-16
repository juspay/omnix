// TODO: Replace this by re-use `omnix-cli`?
use clap::{command, Parser};
use nix_health::{run_checks_with, NixHealth};
use nix_rs::flake::url::FlakeUrl;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Include health checks defined in the given flake
    pub flake_url: Option<FlakeUrl>,

    /// Be quiet by outputting only failed checks
    #[arg(long = "quiet", short = 'q')]
    pub quiet: bool,

    /// Dump the config schema of the health checks (useful when adding them to
    /// a flake.nix)
    #[arg(long = "dump-schema")]
    pub dump_schema: bool,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    human_panic::setup_panic!();
    let args = Args::parse();

    nix_health::logging::setup_logging(args.quiet);

    if args.dump_schema {
        tracing::info!("{}", NixHealth::schema()?);
        return Ok(());
    }

    let checks = run_checks_with(args.flake_url).await?;

    let exit_code = NixHealth::print_report_returning_exit_code(&checks);

    std::process::exit(exit_code)
}
