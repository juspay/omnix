use clap::Parser;
use clap_verbosity_flag::{InfoLevel, Level, Verbosity};
use nix_rs::command::NixCmd;
use nixci::cli::{BuildCommand, Command};

/// Build all outputs of the flake
#[derive(Parser, Debug)]
pub struct CICommand {
    /// Nix command global options
    #[command(flatten)]
    pub nixcmd: NixCmd,

    #[clap(subcommand)]
    command: Option<nixci::cli::Command>,
}

impl CICommand {
    /// Get the command to run
    ///
    /// If the user has not provided one, return the build command by default.
    pub fn command(&self) -> nixci::cli::Command {
        let cmd = BuildCommand::parse_from::<[_; 0], &str>([]);
        let mut cmd = self.command.clone().unwrap_or(Command::Build(cmd));
        cmd.preprocess();
        cmd
    }

    pub async fn run(&self, verbosity: Verbosity<InfoLevel>) -> anyhow::Result<()> {
        nixci::nixci(
            &self.nixcmd,
            &self.command(),
            verbosity.log_level() > Some(Level::Info),
        )
        .await?;
        Ok(())
    }
}
