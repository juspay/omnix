//! Health checks for the user's Nix install

mod caches;
mod max_jobs;

use leptos::*;
use serde::{Deserialize, Serialize};
use tracing::instrument;

use self::caches::Caches;
use self::max_jobs::MaxJobs;
use super::info;

#[instrument(name = "nix-health")]
#[server(GetHealthChecks, "/api")]
pub async fn get_health_checks() -> Result<NixHealth, ServerFnError> {
    let info = info::get_nix_info().await?;
    Ok(NixHealth::new(&info))
}

/// Nix Health check information
///
/// This struct is isomorphic to Vec<Box<&dyn Check>>. We cannot use the latter
/// due to (wasm) serialization limitation with dyn trait objects.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NixHealth {
    max_jobs: MaxJobs,
    caches: Caches,
}

impl NixHealth {
    pub fn new(info: &info::NixInfo) -> Self {
        NixHealth {
            max_jobs: MaxJobs::check(info),
            caches: Caches::check(info),
        }
    }
    pub fn is_healthy(&self) -> bool {
        // TODO: refactor
        let checks: Vec<Box<&dyn Check>> = vec![Box::new(&self.max_jobs), Box::new(&self.caches)];
        checks.iter().all(|check| check.report() == Report::Green)
    }
}

#[derive(Debug, PartialEq, PartialOrd, Eq, Ord, Serialize, Deserialize, Clone)]
pub enum Report {
    Green,
    Red {
        msg: &'static str,
        suggestion: &'static str,
    }, // TODO: Should this be Markdown?
}

impl IntoView for Report {
    fn into_view(self, cx: Scope) -> View {
        view! { cx,
            {match self {
                Report::Green => {
                    view! { cx, <div class="text-green-500">{"✓"}</div> }.into_view(cx)
                }
                Report::Red { msg, suggestion } => {

                    view! { cx,
                        <div class="text-3xl text-red-500">{"✗"}</div>
                        <div class="bg-red-400 rounded bg-border">{msg}</div>
                        <div class="bg-blue-400 rounded bg-border">"Suggestion: " {suggestion}</div>
                    }
                        .into_view(cx)
                }
            }}
        }
        .into_view(cx)
    }
}

pub trait Check: IntoView {
    fn check(info: &info::NixInfo) -> Self
    where
        Self: Sized;

    fn name(&self) -> &'static str;

    fn report(&self) -> Report;
}

impl IntoView for NixHealth {
    fn into_view(self, cx: Scope) -> View {
        view! { cx, <div class="flex justify-start space-x-8">{self.max_jobs} {self.caches}</div> }
            .into_view(cx)
    }
}

#[component]
pub fn ViewCheck<C>(cx: Scope, check: C, children: Children) -> impl IntoView
where
    C: Check + Clone,
{
    view! { cx,
        <div class="bg-white border-2 rounded">
            <h2 class="p-2 text-xl font-bold ">{(&check).name()}</h2>
            <div class="p-2">
                <div class="py-2 bg-base-50">{children(cx)}</div>
                <div class="flex flex-col justify-start space-y-8">{(&check).report()}</div>
            </div>
        </div>
    }
}
