use std::path::PathBuf;

use clap::Parser;

/// Prepare to hack on a flake project
#[derive(Parser, Debug)]
pub struct HackCommand {
    /// Directory of the project
    #[arg(name = "DIR", default_value = ".")]
    dir: PathBuf,
}

impl HackCommand {
    pub async fn run(&self) -> anyhow::Result<()> {
        omnix_hack::core::hack_on(&self.dir).await?;
        Ok(())
    }
}
