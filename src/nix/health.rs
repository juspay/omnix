//! Health checks for the user's Nix install

use leptos::*;
use serde::{Deserialize, Serialize};
use tracing::instrument;

use super::info;

#[derive(Debug, PartialEq, PartialOrd, Eq, Ord, Serialize, Deserialize, Clone)]
pub enum Report {
    Green,
    Red {
        msg: &'static str,
        suggestion: &'static str,
    }, // TODO: Should this be Markdown?
}

#[typetag::serde(tag = "type")]
pub trait Check: dyn_clone::DynClone {
    fn check(info: info::NixInfo) -> Self
    where
        Self: Sized;

    fn report(&self) -> Report;
}

dyn_clone::clone_trait_object!(Check);

#[derive(Debug, PartialEq, PartialOrd, Eq, Ord, Serialize, Deserialize, Clone)]
pub struct MaxJobs(i32);

#[typetag::serde]
impl Check for MaxJobs {
    fn check(info: info::NixInfo) -> Self {
        MaxJobs(info.nix_config.max_jobs.value)
    }
    fn report(&self) -> Report {
        if self > &MaxJobs(1) {
            Report::Green
        } else {
            Report::Red {
                msg: "You are using only 1 core for nix builds",
                suggestion: "Try editing /etc/nix/nix.conf",
            }
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct HealthCheck(Vec<Box<dyn Check>>);

pub fn run_checks(info: info::NixInfo) -> Vec<Box<dyn Check>> {
    vec![Box::new(MaxJobs::check(info))]
}

#[instrument(name = "nix-health")]
#[server(GetHealthChecks, "/api")]
pub async fn get_health_checks() -> Result<HealthCheck, ServerFnError> {
    let info = info::get_nix_info().await?;
    Ok(HealthCheck(run_checks(info)))
}

impl IntoView for HealthCheck {
    fn into_view(self, cx: Scope) -> View {
        let HealthCheck(checks) = self;
        let check0 = checks[0].report();
        view! { cx, <pre>{format!("{:?}", check0)}</pre> }.into_view(cx)
    }
}
