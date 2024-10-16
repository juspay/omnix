//! Health checks for the user's Nix install

pub mod check;
pub mod report;
pub mod traits;

use anyhow::Context;
use colored::Colorize;

use check::direnv::Direnv;
use nix_rs::env::OS;
use nix_rs::flake::url::FlakeUrl;
use nix_rs::{command::NixCmd, info::NixInfo};
use omnix_common::config::{OmConfig, OmConfigError};
use omnix_common::markdown::print_markdown;
use serde::{Deserialize, Serialize};
use tracing::instrument;
use traits::Check;

use self::check::{
    caches::Caches, flake_enabled::FlakeEnabled, max_jobs::MaxJobs, min_nix_version::MinNixVersion,
    rosetta::Rosetta, trusted_users::TrustedUsers,
};

/// Nix Health check information for user's install
///
/// Each field represents an individual check which satisfies the [traits::Checkable] trait.
#[derive(Debug, Default, Serialize, Deserialize, Clone)]
#[serde(default, rename_all = "kebab-case")]
pub struct NixHealth {
    pub max_jobs: MaxJobs,
    pub caches: Caches,
    pub flake_enabled: FlakeEnabled,
    pub nix_version: MinNixVersion,
    pub system: check::system::System,
    pub trusted_users: TrustedUsers,
    pub rosetta: Rosetta,
    pub direnv: Direnv,
}

impl<'a> IntoIterator for &'a NixHealth {
    type Item = &'a dyn traits::Checkable;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    /// Return an iterator to iterate on the fields of [NixHealth]
    fn into_iter(self) -> Self::IntoIter {
        let items: Vec<Self::Item> = vec![
            &self.rosetta,
            &self.nix_version,
            &self.flake_enabled,
            &self.system,
            &self.max_jobs,
            &self.caches,
            &self.trusted_users,
            &self.direnv,
        ];
        items.into_iter()
    }
}

impl NixHealth {
    /// Create [NixHealth] using configuration from the given flake
    ///
    /// Fallback to using the default health check config if the flake doesn't
    /// override it.
    pub async fn from_flake(url: &FlakeUrl) -> Result<Self, OmConfigError> {
        let cmd = NixCmd::get().await;
        let cfg =
            OmConfig::<NixHealth>::from_flake_url(cmd, url, &["om.health", "nix-health"]).await?;
        let (cfg, _rest) = cfg.get_referenced()?;
        Ok(cfg.clone())
    }

    /// Run all checks and collect the results
    #[instrument(skip_all)]
    pub fn run_checks(
        &self,
        nix_info: &nix_rs::info::NixInfo,
        flake_url: Option<FlakeUrl>,
    ) -> Vec<traits::Check> {
        self.into_iter()
            .flat_map(|c| c.check(nix_info, flake_url.as_ref()))
            .collect()
    }

    pub async fn print_report_returning_exit_code(checks: &[traits::Check]) -> anyhow::Result<i32> {
        let mut res = AllChecksResult::new();
        let pwd = std::env::current_dir().unwrap(); // FIXME
        for check in checks {
            match &check.result {
                traits::CheckResult::Green => {
                    tracing::info!("✅ {}", check.title.green().bold());
                    print_markdown(&pwd, &format!("{}", check.info.dimmed())).await?;
                }
                traits::CheckResult::Red { msg, suggestion } => {
                    res.register_failure(check.required);
                    if check.required {
                        print_markdown(&pwd, &format!("❌ {}", check.title.red().bold())).await?;
                    } else {
                        print_markdown(&pwd, &format!("🟧 {}", check.title.yellow().bold()))
                            .await?;
                    }
                    print_markdown(&pwd, &format!("{}", check.info.dimmed())).await?;
                    print_markdown(
                        &pwd,
                        &format!("**Problem**: {}\\\n**Fix**:     {}\n", msg, suggestion),
                    )
                    .await?;
                }
            }
        }
        let code = res.report();
        Ok(code)
    }

    pub fn schema() -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(&NixHealth::default())
    }
}

/// Run health checks, optionally using the given flake's configuration
pub async fn run_checks_with(flake_url: Option<FlakeUrl>) -> anyhow::Result<Vec<Check>> {
    let nix_info = NixInfo::get()
        .await
        .as_ref()
        .with_context(|| "Unable to gather nix info")?;

    let health: NixHealth = match flake_url.as_ref() {
        Some(flake_url) => NixHealth::from_flake(flake_url).await,
        None => Ok(NixHealth::default()),
    }?;

    tracing::info!(
        "🩺️ Checking the health of your Nix setup{}",
        match flake_url.as_ref() {
            Some(flake_url) => format!(" using config from flake '{}'", flake_url),
            None => "".to_string(),
        },
    );
    tracing::info!("   - System: {}", nix_info.nix_config.system.value);
    tracing::info!("   - OS: {}", nix_info.nix_env.os);
    if nix_info.nix_env.os != OS::NixOS {
        tracing::info!("   - Nix installer: {}", nix_info.nix_env.installer);
    }

    let checks = health.run_checks(nix_info, flake_url.clone());
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

    /// Print a summary report of the checks and return the exit code
    fn report(self) -> i32 {
        match self {
            AllChecksResult::Pass => {
                tracing::info!("{}", "✅ All checks passed".green().bold());
                0
            }
            AllChecksResult::PassSomeFail => {
                tracing::warn!(
                    "{}, {}",
                    "✅ Required checks passed".green().bold(),
                    "but some non-required checks failed".yellow().bold()
                );
                0
            }
            AllChecksResult::Fail => {
                tracing::error!("{}", "❌ Some required checks failed".red().bold());
                1
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::check::{caches::Caches, min_nix_version::MinNixVersion};

    #[test]
    fn test_json_deserialize_empty() {
        let json = r#"{}"#;
        let v: super::NixHealth = serde_json::from_str(json).unwrap();
        assert_eq!(v.nix_version, MinNixVersion::default());
        assert_eq!(v.caches, Caches::default());
        println!("{:?}", v);
    }

    #[test]
    fn test_json_deserialize_nix_version() {
        let json = r#"{ "nix-version": { "min-required": "2.17.0" } }"#;
        let v: super::NixHealth = serde_json::from_str(json).unwrap();
        assert_eq!(v.nix_version.min_required.to_string(), "2.17.0");
        assert_eq!(v.caches, Caches::default());
    }

    #[test]
    fn test_json_deserialize_caches() {
        let json = r#"{ "caches": { "required": ["https://foo.cachix.org"] } }"#;
        let v: super::NixHealth = serde_json::from_str(json).unwrap();
        assert_eq!(
            v.caches.required,
            vec![url::Url::parse("https://foo.cachix.org").unwrap()]
        );
        assert_eq!(v.nix_version, MinNixVersion::default());
    }
}
