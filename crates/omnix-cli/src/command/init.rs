use clap::Parser;

#[derive(Parser, Debug)]
pub struct InitConfig {}

impl InitConfig {
    pub async fn run(&self) -> anyhow::Result<()> {
        println!("TODO(om init)");
        Ok(())
    }
}