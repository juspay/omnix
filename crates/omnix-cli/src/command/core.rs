use clap::Subcommand;
use clap_verbosity_flag::{InfoLevel, Verbosity};

#[derive(Subcommand, Debug)]
pub enum Command {
    Show(super::show::ShowCommand),

    Init(super::init::InitCommand),

    Hack(super::hack::HackCommand),

    CI(super::ci::CICommand),

    Health(super::health::HealthCommand),

    Completion(super::completion::CompletionCommand),
}

impl Command {
    pub async fn run(&self, verbosity: Verbosity<InfoLevel>) -> anyhow::Result<()> {
        if !matches!(self, Command::Completion(_)) && !omnix_common::check::nix_installed() {
            tracing::error!("Nix is not installed: https://nixos.asia/en/install");
            std::process::exit(1);
        }

        match self {
            Command::Show(cmd) => cmd.run().await,
            Command::Init(cmd) => cmd.run().await,
            Command::Hack(cmd) => cmd.run().await,
            Command::CI(cmd) => cmd.run(verbosity).await,
            Command::Health(cmd) => cmd.run().await,
            Command::Completion(cmd) => cmd.run(),
        }
    }
}
