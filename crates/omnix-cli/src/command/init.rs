use clap::Parser;

#[derive(Parser, Debug)]
pub struct InitConfig {}

impl InitConfig {
    pub fn run(&self) {
        println!("TODO(om init)");
    }
}
