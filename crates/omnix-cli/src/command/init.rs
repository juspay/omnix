use std::{path::PathBuf, sync::LazyLock};

use clap::Parser;
use nix_rs::flake::url::FlakeUrl;

static REGISTRY: LazyLock<FlakeUrl> =
    LazyLock::new(|| PathBuf::from(env!("OM_INIT_REGISTRY")).into());

/// Initialize a new flake project
#[derive(Parser, Debug)]
pub struct InitCommand {
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

impl InitCommand {
    pub async fn run(&self) -> anyhow::Result<()> {
        tracing::warn!("\n  !! WARNING: `om init` is still under development !!\n");
        flakreate::flakreate(self.registry.clone(), self.path.clone()).await
    }
}
