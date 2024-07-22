use clap::Parser;

/// Build all flake outputs (run CI locally)
#[derive(Parser, Debug)]
pub struct CIConfig {}

impl CIConfig {
    pub async fn run(&self) -> anyhow::Result<()> {
        tracing::info!("TODO(om ci): {:?}", self);
        Ok(())
    }
}
