use std::{path::PathBuf, sync::LazyLock};

use clap::Parser;
use nix_rs::flake::url::FlakeUrl;

static REGISTRY: LazyLock<FlakeUrl> =
    LazyLock::new(|| PathBuf::from(env!("FLAKREATE_REGISTRY")).into());

/// Initialize a new flake project
#[derive(Parser, Debug)]
pub struct InitConfig {
    /// Flake template registry to use
    ///
    /// The flake attribute is treated as a glob pattern to select the
    /// particular template (or subset of templates) to use.
    #[arg(short = 't', default_value_t = REGISTRY.clone())]
    registry: FlakeUrl,

    /// Where to create the template
    #[arg()]
    path: PathBuf,
}

impl InitConfig {
    pub async fn run(&self) -> anyhow::Result<()> {
        flakreate::flakreate(self.registry.clone(), self.path.clone()).await
    }
}
