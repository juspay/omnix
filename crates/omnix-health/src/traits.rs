use colored::Colorize;
use serde::{Deserialize, Serialize};

/// Types that can do specific "health check" for Nix

pub trait Checkable {
    /// Run and create the health check
    ///
    /// NOTE: Some checks may perform impure actions (IO, etc.). Returning an
    /// empty vector indicates that the check is skipped on this environment.
    fn check(
        &self,
        nix_info: &nix_rs::info::NixInfo,
        // The flake against which the check is being run
        //
        // If None, the check is run against the current environment, with no
        // specific configuration from a flake.
        flake: Option<&nix_rs::flake::url::FlakeUrl>,
    ) -> Vec<Check>;
}

/// A health check
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct Check {
    /// A user-facing title of this check
    ///
    /// This value is expected to be unique across all checks.
    pub title: String,

    /// The user-facing information used to conduct this check
    /// TODO: Use Markdown
    pub info: String,

    /// The result of running this check
    pub result: CheckResult,

    /// Whether this check is mandatory
    ///
    /// Failures are considered non-critical if this is false.
    pub required: bool,
}

impl Check {
    /// Log the results using tracing crate
    pub async fn tracing_log(&self) -> anyhow::Result<()> {
        let pwd = std::env::current_dir()?;
        let md = async |s: &str| omnix_common::markdown::render_markdown(&pwd, s).await;
        match &self.result {
            CheckResult::Green => {
                tracing::info!("✅ {}", self.title.green().bold());
                tracing::info!("{}", md(&self.info).await?.dimmed());
            }
            CheckResult::Red { msg, suggestion } => {
                let solution = md(&format!(
                    "**Problem**: {}\\\n**Fix**:     {}\n",
                    msg, suggestion
                ))
                .await?;
                if self.required {
                    tracing::error!("❌ {}", md(&self.title).await?.red().bold());
                    tracing::error!("{}", md(&self.info).await?.dimmed());
                    tracing::error!("{}", solution);
                } else {
                    tracing::warn!("🟧 {}", md(&self.title).await?.yellow().bold());
                    tracing::warn!("{}", md(&self.info).await?.dimmed());
                    tracing::warn!("{}", solution);
                }
            }
        }
        Ok(())
    }
}

/// The result of a health [Check]
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub enum CheckResult {
    /// The check passed
    Green,

    /// The check failed
    Red {
        /// TODO: Use markdown
        msg: String,
        /// TODO: Use markdown
        suggestion: String,
    },
}

impl CheckResult {
    /// When the check is green (ie., healthy)
    pub fn green(&self) -> bool {
        matches!(self, Self::Green)
    }
}
