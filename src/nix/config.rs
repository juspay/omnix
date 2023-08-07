use std::collections::HashMap;

use leptos::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Information about the user's Nix installation
#[serde(rename_all = "kebab-case")]
pub struct NixConfig {
    pub cores: ConfigVal<i32>,
    pub experimental_features: ConfigVal<Vec<String>>,
    pub extra_platforms: ConfigVal<Vec<String>>,
    pub flake_registry: ConfigVal<String>,
    pub max_jobs: ConfigVal<i32>,
    pub max_substitution_jobs: ConfigVal<i32>,
    pub substituters: ConfigVal<Vec<String>>,
    pub system: ConfigVal<String>,
    #[serde(flatten)]
    pub other: HashMap<String, ConfigVal<Value>>,
}

/// The JSON value for a 'nix show-config' key.
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

/// Get the output of `nix show-config`
#[cfg(feature = "ssr")]
pub async fn get_nix_config() -> Result<NixConfig, ServerFnError> {
    run_nix_show_config().await
}

#[cfg(feature = "ssr")]
pub async fn run_nix_show_config() -> Result<NixConfig, ServerFnError> {
    use tokio::process::Command;
    let out = Command::new("nix")
        .args(vec!["show-config", "--json"])
        .output()
        .await?;
    if out.status.success() {
        let v = serde_json::from_slice::<NixConfig>(&out.stdout)?;
        Ok(v)
    } else {
        Err(ServerFnError::ServerError(
            "Unable to determine nix version".into(),
        ))
    }
}

impl IntoView for NixConfig {
    fn into_view(self, cx: Scope) -> View {
        fn mk_row<T: IntoView>(
            cx: Scope,
            key: impl IntoView,
            value: ConfigVal<T>,
        ) -> impl IntoView {
            view! {cx,
                // TODO: Use a nice Tailwind tooltip here, instead of "title"
                // attribute.
                <tr title=value.description>
                    <td class="px-4 py-2 font-bold">{key}</td>
                    // FIXME: The ImplView for Vec<T> renders them side by side.
                    // We should render them as list.
                    <td class="px-4 py-2 text-left">{value.value}</td>
                </tr>
            }
        }
        view! {cx,
            <div class="py-1 my-1 rounded bg-blue-50">
                <table class="text-right">
                    <tbody>
                        {mk_row(cx, "System", self.system)}
                        {mk_row(cx, "Max Jobs", self.max_jobs)}
                        {mk_row(cx, "Cores per build", self.cores)}
                        {mk_row(cx, "Nix Caches", self.substituters)}
                    </tbody>
                </table>
            </div>
        }
        .into_view(cx)
    }
}

#[cfg(feature = "ssr")]
#[tokio::test]
async fn test_run_nix_show_config() {
    let nix_config = run_nix_show_config().await.unwrap();
    println!("Max Jobs: {}", nix_config.max_jobs.value)
}
