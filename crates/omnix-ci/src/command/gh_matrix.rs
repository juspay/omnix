//! The gh-matrix command
use clap::Parser;
use nix_rs::{command::NixCmd, flake::system::System};
use omnix_common::config::OmConfig;

use crate::{config::subflakes::SubflakesConfig, github};

/// Command to generate a Github Actions matrix
#[derive(Parser, Debug, Clone)]
pub struct GHMatrixCommand {
    /// Systems to include in the matrix
    #[arg(long, value_parser, value_delimiter = ',')]
    pub systems: Vec<System>,

    /// Nix command global options
    #[command(flatten)]
    pub nixcmd: NixCmd,
}

impl GHMatrixCommand {
    /// Run the command
    pub async fn run(&self, cfg: OmConfig) -> anyhow::Result<()> {
        let (config, _rest) = cfg.get_sub_config_under::<SubflakesConfig>("ci")?;
        let matrix = github::matrix::GitHubMatrix::from(self.systems.clone(), &config);
        println!("{}", serde_json::to_string(&matrix)?);
        Ok(())
    }
}
