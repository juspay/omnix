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
                if check.required {
                    println!("{}", format!("❌ {}", check.title).red().bold());
                } else {
                    println!("{}", format!("🟧 {}", check.title).yellow().bold());
                }
                println!("   {}", check.info.blue());
                println!("   {}", msg.yellow());
                println!("   {}", suggestion);
            }
        }
        println!();
    }
    let failed_checks: Vec<&Check> = checks
        .iter()
        .filter(|c| matches!(c.result, CheckResult::Red { .. }))
        .collect();
    if !failed_checks.is_empty() {
        let failed_required_checks = failed_checks
            .iter()
            .filter(|c| c.required)
            .collect::<Vec<_>>();
        if !failed_required_checks.is_empty() {
            println!(
                "{}",
                "❌ Some required checks failed (see above)".red().bold()
            );
        } else {
            println!(
                "{}",
                "✅ Some checks passed, but other non-required checks failed"
                    .green()
                    .bold()
            );
        }
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
    let nix_env = NixEnv::detect(flake_url.clone())
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
