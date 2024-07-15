use clap::Parser;

#[derive(Parser, Debug)]
pub struct ShowConfig {}

impl ShowConfig {
    pub async fn run(&self) -> anyhow::Result<()> {
        tracing::info!("TODO(om show)");
        Ok(())
    }
}
