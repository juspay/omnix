//! Rust module for `nix show-config`
use leptos::*;
use serde::{Deserialize, Serialize};
#[cfg(feature = "ssr")]
use tracing::instrument;

/// Nix configuration spit out by `nix show-config`
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct NixConfig {
    pub cores: ConfigVal<i32>,
    pub experimental_features: ConfigVal<Vec<String>>,
    pub extra_platforms: ConfigVal<Vec<String>>,
    pub flake_registry: ConfigVal<String>,
    pub max_jobs: ConfigVal<i32>,
    pub substituters: ConfigVal<Vec<String>>,
    pub system: ConfigVal<String>,
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

/// The HTML view for config values that are lists; rendered as HTML lists.
impl IntoView for ConfigVal<Vec<String>> {
    fn into_view(self, cx: Scope) -> View {
        view! { cx,
            // Render a list of T items in the list 'self'
            <div class="flex flex-col space-y-4">
                {self
                    .value
                    .into_iter()
                    .map(|item| view! { cx, <div>{item}</div> })
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

/// Get the output of `nix show-config`
#[cfg(feature = "ssr")]
#[instrument(name = "show-config")]
pub async fn run_nix_show_config() -> Result<NixConfig, ServerFnError> {
    use tokio::process::Command;
    let mut cmd = Command::new("nix");
    cmd.args(vec![
        "--extra-experimental-features",
        "nix-command",
        "show-config",
        "--json",
    ]);
    let stdout: Vec<u8> = crate::command::run_command(&mut cmd).await?;
    let v = serde_json::from_slice::<NixConfig>(&stdout)?;
    Ok(v)
}

impl IntoView for NixConfig {
    fn into_view(self, cx: Scope) -> View {
        fn mk_row<T: IntoView>(cx: Scope, key: impl IntoView, value: ConfigVal<T>) -> impl IntoView
        where
            ConfigVal<T>: IntoView,
        {
            view! { cx,
                // TODO: Use a nice Tailwind tooltip here, instead of "title"
                // attribute.
                <tr title=&value.description>
                    <td class="px-4 py-2 font-bold">{key}</td>
                    <td class="px-4 py-2 text-left">{value}</td>
                </tr>
            }
        }
        view! { cx,
            <div class="py-1 my-1 rounded bg-blue-50">
                <table class="text-right">
                    <tbody>
                        {mk_row(cx, "System", self.system)} {mk_row(cx, "Max Jobs", self.max_jobs)}
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
