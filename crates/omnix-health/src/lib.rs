//! Health checks for the user's Nix install

pub mod check;
pub mod json;
pub mod report;
pub mod traits;

use anyhow::Context;
use check::shell::ShellCheck;
use colored::Colorize;

use check::direnv::Direnv;
use check::homebrew::Homebrew;
use json::HealthOutput;
use nix_rs::command::NixCmd;
use nix_rs::env::OS;
use nix_rs::flake::url::FlakeUrl;
use nix_rs::info::NixInfo;
use omnix_common::config::{OmConfig, OmConfigError};
use omnix_common::markdown::render_markdown;
use serde::{Deserialize, Serialize};
use tracing::instrument;
use traits::Check;

use self::check::{
    caches::Caches, flake_enabled::FlakeEnabled, max_jobs::MaxJobs, nix_version::NixVersionCheck,
    rosetta::Rosetta, trusted_users::TrustedUsers,
};

/// Nix Health check of user's install
///
/// Each check field is expected to implement [traits::Checkable].
#[derive(Debug, Default, Serialize, Deserialize, Clone)]
#[serde(default, rename_all = "kebab-case")]
pub struct NixHealth {
    pub flake_enabled: FlakeEnabled,
    pub nix_version: NixVersionCheck,
    pub rosetta: Rosetta,
    pub max_jobs: MaxJobs,
    pub trusted_users: TrustedUsers,
    pub caches: Caches,
    pub direnv: Direnv,
    pub homebrew: Homebrew,
    pub shell: ShellCheck,
}

/// Convert [NixHealth] into a generic [Vec] of checks
impl<'a> IntoIterator for &'a NixHealth {
    type Item = &'a dyn traits::Checkable;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    /// Return an iterator to iterate on the fields of [NixHealth]
    fn into_iter(self) -> Self::IntoIter {
        let items: Vec<Self::Item> = vec![
            &self.flake_enabled,
            &self.nix_version,
            &self.rosetta,
            &self.max_jobs,
            &self.trusted_users,
            &self.caches,
            &self.direnv,
            &self.homebrew,
            &self.shell,
        ];
        items.into_iter()
    }
}

impl NixHealth {
    /// Create [NixHealth] using configuration from the given flake
    ///
    /// Fallback to using the default health check config if the flake doesn't
    /// override it.
    pub fn from_om_config(om_config: &OmConfig) -> Result<Self, OmConfigError> {
        let (cfg, _rest) = om_config.get_sub_config_under::<Self>("health")?;
        Ok(cfg.clone())
    }

    /// Run all checks and collect the results
    #[instrument(skip_all)]
    pub fn run_all_checks(
        &self,
        nix_info: &NixInfo,
        flake_url: Option<FlakeUrl>,
    ) -> Vec<(&'static str, Check)> {
        self.into_iter()
            .flat_map(|c| c.check(nix_info, flake_url.as_ref()))
            .collect()
    }

    pub async fn print_report_returning_exit_code(
        checks: &Vec<(&'static str, Check)>,
        json_only: bool,
    ) -> anyhow::Result<i32> {
        let mut res = AllChecksResult::new();
        for (_, check) in checks {
            if !json_only {
                check.tracing_log().await?;
            }
            if !check.result.green() {
                res.register_failure(check.required);
            };
        }

        let code = res.report();

        if json_only {
            let json = HealthOutput::get(checks.to_vec()).await?;
            println!("{}", serde_json::to_string(&json)?);
        }
        Ok(code)
    }

    pub fn schema() -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(&NixHealth::default())
    }
}

/// Run all health checks, optionally using the given flake's configuration
pub async fn run_all_checks_with(
    nixcmd: &NixCmd,
    flake_url: Option<FlakeUrl>,
    json_only: bool,
) -> anyhow::Result<Vec<(&'static str, Check)>> {
    let nix_info = NixInfo::get()
        .await
        .as_ref()
        .with_context(|| "Unable to gather nix info")?;

    let health: NixHealth = match flake_url.as_ref() {
        Some(flake_url) => {
            let om_config = OmConfig::get(nixcmd, flake_url).await?;
            NixHealth::from_om_config(&om_config)
        }
        None => Ok(NixHealth::default()),
    }?;

    tracing::info!(
        "🩺️ Checking the health of your Nix setup (flake: '{}')",
        match flake_url.as_ref() {
            Some(url) => url.to_string(),
            None => "N/A".to_string(),
        }
    );

    if !json_only {
        print_info_banner(flake_url.as_ref(), nix_info).await?;
    }

    let checks = health.run_all_checks(nix_info, flake_url);
    Ok(checks)
}

async fn print_info_banner(flake_url: Option<&FlakeUrl>, nix_info: &NixInfo) -> anyhow::Result<()> {
    let pwd = std::env::current_dir()?;

    let mut table = String::from("| Property | Value |\n|----------|-------|\n");
    table.push_str(&format!(
        "| Flake | {} |\n",
        match flake_url {
            Some(url) => url.to_string(),
            None => "N/A".to_string(),
        }
    ));
    table.push_str(&format!(
        "| System | {} |\n",
        nix_info.nix_config.system.value
    ));
    table.push_str(&format!("| OS | {} |\n", nix_info.nix_env.os));
    if nix_info.nix_env.os != OS::NixOS {
        table.push_str(&format!(
            "| Nix installer | {} |\n",
            nix_info.nix_env.installer
        ));
    }
    table.push_str(&format!("| RAM | {:?} |\n", nix_info.nix_env.total_memory));
    table.push_str(&format!(
        "| Disk Space | {:?} |",
        nix_info.nix_env.total_disk_space
    ));

    tracing::info!("{}", render_markdown(&pwd, &table).await?);
    Ok(())
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
    use std::str::FromStr;

    use nix_rs::version_spec::NixVersionReq;

    use crate::check::{caches::Caches, nix_version::NixVersionCheck};

    #[test]
    fn test_json_deserialize_empty() {
        let json = r#"{}"#;
        let v: super::NixHealth = serde_json::from_str(json).unwrap();
        assert_eq!(v.nix_version, NixVersionCheck::default());
        assert_eq!(v.caches, Caches::default());
        println!("{:?}", v);
    }

    #[test]
    fn test_json_deserialize_nix_version() {
        let json = r#"{ "nix-version": { "supported": ">=2.17.0" } }"#;
        let v: super::NixHealth = serde_json::from_str(json).unwrap();
        assert_eq!(
            v.nix_version.supported,
            NixVersionReq::from_str(">=2.17.0").unwrap()
        );
        assert_eq!(v.caches, Caches::default());
    }

    #[test]
    fn test_json_deserialize_caches() {
        let json = r#"{ "caches": { "required": ["https://foo.cachix.org"] } }"#;
        let v: super::NixHealth = serde_json::from_str(json).unwrap();
        assert_eq!(
            v.caches.required,
            vec!["https://foo.cachix.org".to_string()]
        );
        assert_eq!(v.nix_version, NixVersionCheck::default());
    }
}
