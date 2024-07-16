use anyhow::Context;
use clap::{command, Parser};
use nix_health::{traits::Check, NixHealth};
use nix_rs::{command::NixCmd, env::NixEnv, flake::url::FlakeUrl, info::NixInfo};

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

    let checks = run_checks(args.flake_url).await?;

    let exit_code = NixHealth::print_report_returning_exit_code(&checks);

    std::process::exit(exit_code)
}

/// Run health checks, taking current directory flake into account if there is
/// one.
async fn run_checks(flake_url: Option<FlakeUrl>) -> anyhow::Result<Vec<Check>> {
    let nix_info = NixInfo::from_nix(&NixCmd::default())
        .await
        .with_context(|| "Unable to gather nix info")?;
    let nix_env = NixEnv::detect()
        .await
        .with_context(|| "Unable to gather system info")?;
    let action_msg = format!(
        "🩺️ Checking the health of your Nix setup ({} on {})",
        &nix_info.nix_config.system.value, &nix_env.os
    );
    let health: NixHealth = match flake_url.as_ref() {
        Some(flake_url) => {
            tracing::info!("{}, using config from flake '{}':", action_msg, flake_url);
            NixHealth::from_flake(flake_url).await
        }
        None => {
            tracing::info!("{}:", action_msg);
            Ok(NixHealth::default())
        }
    }?;
    let checks = health.run_checks(&nix_info, flake_url.clone());
    Ok(checks)
}
