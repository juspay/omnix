use clap::Parser;
use nix_health::{run_checks_with, NixHealth};
use nix_rs::flake::url::FlakeUrl;

#[derive(Parser, Debug)]
pub struct HealthConfig {
    /// Include health checks defined in the given flake
    pub flake_url: Option<FlakeUrl>,

    /// Dump the config schema of the health checks (useful when adding them to
    /// a flake.nix)
    #[arg(long = "dump-schema")]
    pub dump_schema: bool,
}

impl HealthConfig {
    pub async fn run(&self) -> anyhow::Result<()> {
        if self.dump_schema {
            tracing::info!("{}", NixHealth::schema()?);
            return Ok(());
        }
        // TODO: Setup logging (unify `crates/nix_health/src/logging.rs` and `crates/omnix/src/logging.rs`)
        // TODO: `om.health` config?
        let checks = run_checks_with(self.flake_url.clone()).await?;
        let exit_code = NixHealth::print_report_returning_exit_code(&checks);
        if exit_code != 0 {
            std::process::exit(exit_code);
        }
        Ok(())
    }
}
