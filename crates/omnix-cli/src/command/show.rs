use clap::Parser;

#[derive(Parser, Debug)]
pub struct ShowConfig {}

impl ShowConfig {
    pub fn run(&self) {
        println!("TODO(om show)");
    }
}
