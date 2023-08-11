//! Health checks for the user's Nix install

use leptos::*;
use serde::{Deserialize, Serialize};
use tracing::instrument;

use super::info;

#[derive(Debug, PartialEq, PartialOrd, Eq, Ord, Serialize, Deserialize, Clone)]
pub struct NixHealthChecks {
    max_jobs: MaxJobs,
}

impl NixHealthChecks {
    pub fn new(info: info::NixInfo) -> Self {
        NixHealthChecks {
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

    fn report(&self) -> Report;
}

#[derive(Debug, PartialEq, PartialOrd, Eq, Ord, Serialize, Deserialize, Clone)]
pub struct MaxJobs(i32);

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

#[instrument(name = "nix-health")]
#[server(GetHealthChecks, "/api")]
pub async fn get_health_checks() -> Result<NixHealthChecks, ServerFnError> {
    let info = info::get_nix_info().await?;
    Ok(NixHealthChecks::new(info))
}

impl IntoView for NixHealthChecks {
    fn into_view(self, cx: Scope) -> View {
        let checks = self.as_list();
        let check0 = checks[0].report();
        view! { cx, <pre>{format!("{:?}", check0)}</pre> }.into_view(cx)
    }
}
