use clap::Parser;
use clap_verbosity_flag::{InfoLevel, Level, Verbosity};
use nix_rs::command::NixCmd;

/// Build all flake outputs (run CI locally)
#[derive(Parser, Debug)]
pub struct CIConfig {
    /// Nix command global options
    #[command(flatten)]
    pub nixcmd: NixCmd,

    #[clap(subcommand)]
    pub command: nixci::cli::Command,
}

impl CIConfig {
    pub async fn run(&self, verbosity: Verbosity<InfoLevel>) -> anyhow::Result<()> {
        nixci::nixci(
            &self.nixcmd,
            &self.command,
            verbosity.log_level() > Some(Level::Info),
        )
        .await?;
        Ok(())
    }
}
