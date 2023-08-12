//! Health checks for the user's Nix install

mod caches;
mod max_jobs;

use leptos::*;
use serde::{Deserialize, Serialize};
use tracing::instrument;

use self::caches::Caches;
use self::max_jobs::MaxJobs;
use super::info;

/// Nix Health check information
///
/// This struct is isomorphic to Vec<Box<&dyn Check>>. We cannot use the latter
/// due to (wasm) serialization limitation with dyn trait objects.
#[derive(Debug, PartialEq, PartialOrd, Eq, Ord, Serialize, Deserialize, Clone)]
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
    pub fn as_list(&self) -> Vec<Box<&dyn Check>> {
        vec![Box::new(&self.max_jobs), Box::new(&self.caches)]
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

pub trait Check {
    fn check(info: &info::NixInfo) -> Self
    where
        Self: Sized;

    fn name(&self) -> &'static str;

    fn report(&self) -> Report;
}

#[instrument(name = "nix-health")]
#[server(GetHealthChecks, "/api")]
pub async fn get_health_checks() -> Result<NixHealth, ServerFnError> {
    let info = info::get_nix_info().await?;
    Ok(NixHealth::new(&info))
}

impl IntoView for NixHealth {
    fn into_view(self, cx: Scope) -> View {
        let checks = self.as_list();
        view! { cx,
            <div class="flex justify-start space-x-8">
                {checks
                    .into_iter()
                    .map(|check| {
                        view! { cx,
                            <div class="p-2 border-2 rounded bg-primary-50">
                                <b>{check.name()}</b>
                                <div class="flex flex-col justify-start space-y-8">
                                    {match check.report() {
                                        Report::Green => {
                                            view! { cx, <div class="text-green-500">{"✓"}</div> }
                                                .into_view(cx)
                                        }
                                        Report::Red { msg, suggestion } => {

                                            view! { cx,
                                                <div class="text-red-500">{"✗"}</div>
                                                <div class="bg-red-400 rounded bg-border">{msg}</div>
                                                <div class="bg-blue-400 rounded bg-border">
                                                    "Suggestion: " {suggestion}
                                                </div>
                                            }
                                                .into_view(cx)
                                        }
                                    }}

                                </div>
                            </div>
                        }
                            .into_view(cx)
                    })
                    .collect_view(cx)}

            </div>
        }
        .into_view(cx)
    }
}
