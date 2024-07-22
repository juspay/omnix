use clap::Subcommand;

pub mod ci;
pub mod health;
pub mod init;
pub mod show;

#[derive(Subcommand, Debug)]
pub enum Command {
    Show(show::ShowConfig),

    Init(init::InitConfig),

    CI(ci::CIConfig),

    Health(health::HealthConfig),
}

impl Command {
    pub async fn run(&self) -> anyhow::Result<()> {
        match self {
            Command::Show(config) => config.run().await,
            Command::Init(config) => config.run().await,
            Command::CI(config) => config.run().await,
            Command::Health(config) => config.run().await,
        }
    }
}
