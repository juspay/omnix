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

    /// Be quiet by outputting only failed checks
    #[arg(long = "quiet", short = 'q')]
    pub quiet: bool,

    /// Dump the config schema of the health checks (useful when adding them to
    /// a flake.nix)
    #[arg(long = "dump-schema")]
    pub dump_schema: bool,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    human_panic::setup_panic!();
    let args = Args::parse();
    if args.dump_schema {
        println!("{}", NixHealth::schema()?);
        return Ok(());
    }

    let flake_url = args
        .flake_url
        .map(|url| url.with_fully_qualified_root_attr("nix-health"));
    let checks = run_checks(flake_url).await?;
    let mut res = AllChecksResult::new();
    for check in &checks {
        match &check.result {
            CheckResult::Green => {
                if !args.quiet {
                    println!("{}", format!("✅ {}", check.title).green().bold());
                    println!("   {}", check.info.blue());
                }
            }
            CheckResult::Red { msg, suggestion } => {
                res.register_failure(check.required);
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
    }
    std::process::exit(res.report())
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
    let action_msg = format!(
        "🩺️ Checking the health of your Nix setup ({} on {})",
        &nix_info.nix_config.system.value, &nix_env.os
    );
    let health: NixHealth = match flake_url.as_ref() {
        Some(flake_url) => {
            println!("{}, using config from flake '{}':", action_msg, flake_url);
            NixHealth::from_flake(flake_url.clone()).await
        }
        None => {
            println!("{}:", action_msg);
            Ok(NixHealth::default())
        }
    }?;
    let checks = health.run_checks(&nix_info, &nix_env, flake_url.clone());
    Ok(checks)
}

/// A convenient type to aggregate check failures, and summary report at end.
enum AllChecksResult {
    Pass,
    PassSomeFail,
    Fail,
}

impl AllChecksResult {
    fn new() -> Self {
        AllChecksResult::Pass
    }

    fn register_failure(&mut self, required: bool) {
        if required {
            *self = AllChecksResult::Fail;
        } else if matches!(self, AllChecksResult::Pass) {
            *self = AllChecksResult::PassSomeFail;
        }
    }

    fn report(self) -> i32 {
        match self {
            AllChecksResult::Pass => {
                println!("{}", "✅ All checks passed".green().bold());
                0
            }
            AllChecksResult::PassSomeFail => {
                println!(
                    "{}, {}",
                    "✅ Required checks passed".green().bold(),
                    "but some non-required checks failed".yellow().bold()
                );
                0
            }
            AllChecksResult::Fail => {
                println!("{}", "❌ Some required checks failed".red().bold());
                1
            }
        }
    }
}
