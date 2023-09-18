use anyhow::Context;
use clap::{command, Parser};
use colored::Colorize;
use nix_health::{
    traits::{Check, CheckResult},
    NixHealth,
};
use nix_rs::{command::NixCmd, env::NixEnv, flake::url::FlakeUrl, info::NixInfo};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Include health checks defined in the given flake
    pub flake_url: Option<FlakeUrl>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    human_panic::setup_panic!();
    let args = Args::parse();
    let flake_url = args
        .flake_url
        .map(|url| url.with_fully_qualified_root_attr("nix-health"));
    let checks = run_checks(flake_url).await?;
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
async fn run_checks(flake_url: Option<FlakeUrl>) -> anyhow::Result<Vec<Check>> {
    let nix_info = NixInfo::from_nix(&NixCmd::default())
        .await
        .with_context(|| "Unable to gather nix info")?;
    let nix_env = NixEnv::detect()
        .await
        .with_context(|| "Unable to gather system info")?;
    let health: NixHealth = match flake_url {
        Some(flake_url) => {
            println!(
                "🩺️ Checking the health of your Nix setup, using config from flake '{}':\n",
                flake_url
            );
            NixHealth::from_flake(flake_url).await
        }
        None => {
            println!("🩺️️ Checking the health of your Nix setup:\n");
            Ok(NixHealth::default())
        }
    }?;
    let checks = health.run_checks(&nix_info, &nix_env);
    Ok(checks)
}
