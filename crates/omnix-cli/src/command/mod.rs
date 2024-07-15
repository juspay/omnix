use clap::Subcommand;

pub mod init;
pub mod show;

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Inspect a flake
    Show(show::ShowConfig),

    /// Initialize a flake
    Init(init::InitConfig),
}

impl Command {
    pub fn run(&self) {
        match self {
            Command::Show(config) => config.run(),
            Command::Init(config) => config.run(),
        }
    }
}
