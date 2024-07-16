use clap::Parser;

#[derive(Parser, Debug)]
pub struct HealthConfig {}

impl HealthConfig {
    pub async fn run(&self) -> anyhow::Result<()> {
        tracing::info!("TODO(om health): {:?}", self);
        Ok(())
    }
}
