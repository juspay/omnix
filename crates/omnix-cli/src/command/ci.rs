use clap::Parser;
use clap_verbosity_flag::{InfoLevel, Level, Verbosity};
use nix_rs::command::NixCmd;
use nixci::command::{build::BuildCommand, core::Command};

/// Build all outputs of the flake
#[derive(Parser, Debug)]
pub struct CICommand {
    /// Nix command global options
    #[command(flatten)]
    pub nixcmd: NixCmd,

    #[clap(subcommand)]
    command: Option<Command>,
}

impl CICommand {
    /// Get the command to run
    ///
    /// If the user has not provided one, return the build command by default.
    pub fn command(&self) -> Command {
        let mut cmd = self.command.clone().unwrap_or_default();
        cmd.preprocess();
        cmd
    }

    pub async fn run(&self, verbosity: Verbosity<InfoLevel>) -> anyhow::Result<()> {
        self.command()
            .run(&self.nixcmd, verbosity.log_level() > Some(Level::Info))
            .await?;
        Ok(())
    }
}
