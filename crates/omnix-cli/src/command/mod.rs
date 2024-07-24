use clap::Subcommand;

use crate::Args;
pub mod ci;
pub mod health;
pub mod init;
pub mod show;

use anyhow::Ok;
use clap::CommandFactory;
use clap_complete::generate;
use std::io;

#[derive(Subcommand, Debug)]
pub enum Command {
    Show(show::ShowConfig),

    Init(init::InitConfig),

    CI(ci::CIConfig),

    Health(health::HealthConfig),

    /// Generates shell completion scripts
    Completion {
        #[arg(value_enum)]
        shell: clap_complete::Shell,
    },
}

impl Command {
    pub async fn run(&self) -> anyhow::Result<()> {
        match self {
            Command::Show(config) => config.run().await,
            Command::Init(config) => config.run().await,
            Command::CI(config) => config.run().await,
            Command::Health(config) => config.run().await,
            Command::Completion { shell } => {
                let mut cli = Args::command();
                generate(*shell, &mut cli, "om", &mut io::stdout());
                Ok(())
            }
        }
    }
}
