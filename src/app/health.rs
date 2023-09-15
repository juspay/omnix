//! Nix health check UI

use leptos::*;
use leptos_extra::query::{self, RefetchQueryButton};
use leptos_meta::*;
use nix_health::traits::{Check, CheckResult};
use tracing::instrument;

use crate::widget::*;

/// Nix health checks
#[component]
pub fn NixHealthRoute(cx: Scope) -> impl IntoView {
    let title = "Nix Health";
    let result = query::use_server_query(cx, || (), get_nix_health);
    let data = result.data;
    view! { cx,
        <Title text=title/>
        <h1 class="text-5xl font-bold">{title}</h1>
        <RefetchQueryButton result query=|| ()/>
        <div class="my-1">
            <SuspenseWithErrorHandling>
                <div class="flex flex-col items-stretch justify-start space-y-8 text-left">
                    <For
                        each=move || data.get().unwrap_or(Ok(vec![])).unwrap_or(vec![])
                        key=|check| check.title.clone()
                        view=move |cx, check| {
                            view! { cx, <ViewCheck check/> }
                        }
                    />

                </div>

            </SuspenseWithErrorHandling>
        </div>
    }
}

#[component]
fn ViewCheck(cx: Scope, check: Check) -> impl IntoView {
    view! { cx,
        <div class="contents">
            <details
                open=check.result != CheckResult::Green
                class="my-2 bg-white border-2 rounded-lg cursor-pointer hover:bg-primary-100 border-base-300"
            >
                <summary class="p-4 text-xl font-bold">
                    <CheckResultSummaryView green=check.result.green()/>
                    {" "}
                    {check.title}
                </summary>
                <div class="p-4">
                    <div class="p-2 my-2 font-mono text-sm bg-black text-base-100">
                        {check.info}
                    </div>
                    <div class="flex flex-col justify-start space-y-4">
                        {match check.result {
                            CheckResult::Green => view! { cx, "" }.into_view(cx),
                            CheckResult::Red { msg, suggestion } => {
                                view! { cx,
                                    <h3 class="my-2 font-bold text-l"></h3>
                                    <div class="p-2 bg-red-400 rounded bg-border">{msg}</div>
                                    <h3 class="my-2 font-bold text-l"></h3>
                                    <div class="p-2 bg-blue-400 rounded bg-border">
                                        {suggestion}
                                    </div>
                                }
                                    .into_view(cx)
                            }
                        }}

                    </div>
                </div>
            </details>
        </div>
    }
}

#[component]
pub fn CheckResultSummaryView(cx: Scope, green: bool) -> impl IntoView {
    if green {
        view! { cx, <span class="text-green-500">{"✓"}</span> }
    } else {
        view! { cx, <span class="text-red-500">{"✗"}</span> }
    }
}

/// Get [NixHealth] information
#[instrument(name = "nix-health")]
#[server(GetNixHealth, "/api")]
pub async fn get_nix_health(_unit: ()) -> Result<Vec<nix_health::traits::Check>, ServerFnError> {
    use nix_health::NixHealth;
    use nix_rs::{env, info};
    let nix_info = info::NixInfo::from_nix(&nix_rs::command::NixCmd::default()).await?;
    let nix_env = env::NixEnv::detect().await?;
    let health = NixHealth::default();
    let checks = health.run_checks(&nix_info, &nix_env);
    Ok(checks)
}
