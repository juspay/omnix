use clap::Parser;
use omnix_ci::command::core::Command;

/// Build all outputs of the flake
#[derive(Parser, Debug)]
pub struct CICommand {
    #[clap(subcommand)]
    command: Option<Command>,
}

impl CICommand {
    /// Run this sub-command
    pub async fn run(&self) -> anyhow::Result<()> {
        self.command().run().await?;
        Ok(())
    }

    /// Get the command to run
    ///
    /// If the user has not provided one, return the build command by default.
    fn command(&self) -> Command {
        self.command.clone().unwrap_or_default()
    }
}
