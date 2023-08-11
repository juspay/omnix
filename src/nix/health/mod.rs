//! Health checks for the user's Nix install

mod max_jobs;

use leptos::*;
use serde::{Deserialize, Serialize};
use tracing::instrument;

use self::max_jobs::MaxJobs;
use super::info;

/// Nix Health check information
///
/// This struct is isomorphic to Vec<Box<&dyn Check>>. We cannot use the latter
/// due to (wasm) serialization limitation with dyn trait objects.
#[derive(Debug, PartialEq, PartialOrd, Eq, Ord, Serialize, Deserialize, Clone)]
pub struct NixHealth {
    max_jobs: MaxJobs,
}

impl NixHealth {
    pub fn new(info: info::NixInfo) -> Self {
        NixHealth {
            max_jobs: MaxJobs::check(info),
        }
    }
    pub fn as_list(&self) -> Vec<Box<&dyn Check>> {
        vec![Box::new(&self.max_jobs)]
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
    fn check(info: info::NixInfo) -> Self
    where
        Self: Sized;

    fn name(&self) -> &'static str;

    fn report(&self) -> Report;
}

#[instrument(name = "nix-health")]
#[server(GetHealthChecks, "/api")]
pub async fn get_health_checks() -> Result<NixHealth, ServerFnError> {
    let info = info::get_nix_info().await?;
    Ok(NixHealth::new(info))
}

impl IntoView for NixHealth {
    fn into_view(self, cx: Scope) -> View {
        let checks = self.as_list();
        view! { cx,
            {checks
                .into_iter()
                .map(|check| {
                    view! { cx,
                        <b>{check.name()}</b>
                        {match check.report() {
                            Report::Green => {
                                view! { cx, <span class="text-green-500">{"✓"}</span> }
                                    .into_view(cx)
                            }
                            Report::Red { msg, suggestion } => {

                                view! { cx,
                                    <span class="text-red-500">{"✗"}</span>
                                    <div class="red">{msg}</div>
                                    <div class="red">"Suggestion: " {suggestion}</div>
                                }
                                    .into_view(cx)
                            }
                        }}
                    }
                        .into_view(cx)
                })
                .collect::<Vec<_>>()}
        }
        .into_view(cx)
    }
}
