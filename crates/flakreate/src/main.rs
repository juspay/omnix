use clap::Parser;
use flakreate::{flakreate, Args};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    if args.verbose {
        println!("DEBUG {args:?}");
    }
    flakreate(args.registry, args.path).await
}
