//! The gh-matrix command
use clap::Parser;
use nix_rs::flake::system::System;
use omnix_common::config::OmConfig;

use crate::{config::subflakes::SubflakesConfig, flake_ref::FlakeRef, github};

/// Command to generate a Github Actions matrix
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
    /// Run the command
    pub async fn run(&self, cfg: OmConfig<SubflakesConfig>) -> anyhow::Result<()> {
        let matrix = github::matrix::GitHubMatrix::from(self.systems.clone(), &cfg.selected_config);
        println!("{}", serde_json::to_string(&matrix)?);
        Ok(())
    }
}
