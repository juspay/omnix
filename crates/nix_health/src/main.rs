use std::path::Path;

use anyhow::Context;
use colored::Colorize;
use nix_health::{
    traits::{Check, CheckResult},
    NixHealth,
};
use nix_rs::{command::NixCmd, env::NixEnv, info::NixInfo};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    human_panic::setup_panic!();
    let checks = run_checks().await?;
    println!("Checking the health of your Nix setup:\n");
    for check in &checks {
        match &check.result {
            CheckResult::Green => {
                println!("{}", format!("✅ {}", check.title).green().bold());
                println!("   {}", check.info.blue());
            }
            CheckResult::Red { msg, suggestion } => {
                println!("{}", format!("❌ {}", check.title).red().bold());
                println!("   {}", check.info.blue());
                println!("   {}", msg.yellow());
                println!("   {}", suggestion);
            }
        }
        println!();
    }
    if checks
        .iter()
        .any(|c| matches!(c.result, CheckResult::Red { .. }))
    {
        println!("{}", "!! Some checks failed (see above)".red().bold());
        std::process::exit(1);
    } else {
        println!("{}", "✅ All checks passed".green().bold());
        Ok(())
    }
}

/// Run health checks, taking current directory flake into account if there is
/// one.
async fn run_checks() -> anyhow::Result<Vec<Check>> {
    let nix_info = NixInfo::from_nix(&NixCmd::default())
        .await
        .with_context(|| "Unable to gather nix info")?;
    let nix_env = NixEnv::detect()
        .await
        .with_context(|| "Unable to gather system info")?;
    let health: NixHealth = if Path::new("flake.nix").exists() {
        NixHealth::from_flake(".#nix-health.default".into()).await
    } else {
        Ok(NixHealth::default())
    }?;
    let checks = health.run_checks(&nix_info, &nix_env);
    Ok(checks)
}
