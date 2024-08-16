use clap::Subcommand;
use clap_verbosity_flag::{InfoLevel, Verbosity};

pub mod ci;
mod completion;
pub mod health;
pub mod init;
pub mod show;

#[derive(Subcommand, Debug)]
pub enum Command {
    Show(show::ShowCommand),

    Init(init::InitCommand),

    CI(ci::CICommand),

    Health(health::HealthCommand),

    Completion(completion::CompletionCommand),
}

impl Command {
    pub async fn run(&self, verbosity: Verbosity<InfoLevel>) -> anyhow::Result<()> {
        match self {
            Command::Completion(_cmd) => (),
            _ => {
                let nix_installed = omnix_common::check::nix_installed();
                if !nix_installed {
                    tracing::error!(
                        "Nix is not installed. Please install Nix: https://nixos.asia/en/install"
                    );
                    std::process::exit(1);
                }
            }
        }

        match self {
            Command::Show(cmd) => cmd.run().await,
            Command::Init(cmd) => cmd.run().await,
            Command::CI(cmd) => cmd.run(verbosity).await,
            Command::Health(cmd) => cmd.run().await,
            Command::Completion(cmd) => cmd.run(),
        }
    }
}
