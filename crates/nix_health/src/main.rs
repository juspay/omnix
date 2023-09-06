use anyhow::Context;
use colored::Colorize;
use nix_health::{report::Report, traits::Check, NixHealth};
use nix_rs::{command::NixCmd, info::NixInfo};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    human_panic::setup_panic!();
    let info = NixInfo::from_nix(&NixCmd::default())
        .await
        .with_context(|| "Unable to gather nix info")?;
    let health = NixHealth::check(&info);
    for check in &health {
        let report = check.report();
        match report {
            Report::Green => {
                println!("{}", format!("✅ {}", check.name()).green().bold());
            }
            Report::Red(details) => {
                println!("{}", format!("❌ {}", check.name()).red().bold());
                println!("   {}", details.msg.yellow());
                println!("   {}", details.suggestion);
            }
        }
    }
    Ok(())
}
