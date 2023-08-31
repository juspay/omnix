//! Rust module for `nix show-config`
use std::fmt::Display;

use leptos::*;
use serde::{Deserialize, Serialize};
#[cfg(feature = "ssr")]
use tracing::instrument;
use url::Url;

use super::flake::system::System;

/// Nix configuration spit out by `nix show-config`
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct NixConfig {
    pub cores: ConfigVal<i32>,
    pub experimental_features: ConfigVal<Vec<String>>,
    pub extra_platforms: ConfigVal<Vec<String>>,
    pub flake_registry: ConfigVal<String>,
    pub max_jobs: ConfigVal<i32>,
    pub substituters: ConfigVal<Vec<Url>>,
    pub system: ConfigVal<System>,
}

/// The value for each 'nix show-config --json' key.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfigVal<T> {
    /// Current value in use.
    pub value: T,
    /// Default value by Nix.
    pub default_value: T,
    /// Description of this config item.
    pub description: String,
}

impl NixConfig {
    /// Get the output of `nix show-config`
    #[cfg(feature = "ssr")]
    #[instrument(name = "show-config")]
    pub async fn from_nix(
        nix_cmd: &super::command::NixCmd,
    ) -> Result<NixConfig, super::command::NixCmdError> {
        let v = nix_cmd
            .run_with_args_expecting_json(&["show-config", "--json"])
            .await?;
        Ok(v)
    }
}

/// The HTML view for config values that are lists; rendered as HTML lists.
impl<T> IntoView for ConfigVal<Vec<T>>
where
    T: Display,
{
    fn into_view(self, cx: Scope) -> View {
        view! { cx,
            // Render a list of T items in the list 'self'
            <div class="flex flex-col space-y-4">
                {self
                    .value
                    .into_iter()
                    .map(|item| view! { cx, <li class="list-disc">{item.to_string()}</li> })
                    .collect_view(cx)}
            </div>
        }
        .into_view(cx)
    }
}

impl IntoView for ConfigVal<i32> {
    fn into_view(self, cx: Scope) -> View {
        self.value.into_view(cx)
    }
}

impl IntoView for ConfigVal<String> {
    fn into_view(self, cx: Scope) -> View {
        self.value.into_view(cx)
    }
}

impl IntoView for ConfigVal<System> {
    fn into_view(self, cx: Scope) -> View {
        self.value.to_string().into_view(cx)
    }
}

#[cfg(feature = "ssr")]
#[tokio::test]
async fn test_nix_config() {
    let v = NixConfig::from_nix(&crate::command::NixCmd::default()).await?;
    println!("Max Jobs: {}", v.max_jobs.value)
}
