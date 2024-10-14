use clap::Parser;

/// Prepare to hack on a flake project
#[derive(Parser, Debug)]
pub struct HackCommand {}

impl HackCommand {
    pub async fn run(&self) -> anyhow::Result<()> {
        omnix_hack::core::hack_on().await?;
        Ok(())
    }
}
