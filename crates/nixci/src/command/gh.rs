use clap::Parser;
use nix_rs::flake::system::System;

use crate::{config::core::Config, flake_ref::FlakeRef, github};

#[derive(Parser, Debug, Clone)]
pub struct GHMatrixCommand {
    /// Flake URL or github URL
    ///
    /// A specific nixci configuration can be specified
    /// using '#': e.g. `nixci .#extra-tests`
    #[arg(default_value = ".")]
    pub flake_ref: FlakeRef,

    /// Systems to include in the matrix
    #[arg(long, value_parser, value_delimiter = ',')]
    pub systems: Vec<System>,
}

impl GHMatrixCommand {
    pub async fn run(&self, cfg: Config) -> anyhow::Result<()> {
        let matrix = github::matrix::GitHubMatrix::from(self.systems.clone(), &cfg.subflakes);
        println!("{}", serde_json::to_string(&matrix)?);
        Ok(())
    }
}
