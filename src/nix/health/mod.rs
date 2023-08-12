//! Health checks for the user's Nix install

mod check;
mod report;
mod traits;

use leptos::*;
use serde::{Deserialize, Serialize};
use tracing::instrument;

use self::check::caches::Caches;
use self::check::max_jobs::MaxJobs;
use self::report::Report;
use self::traits::Check;
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

impl IntoView for NixHealth {
    fn into_view(self, cx: Scope) -> View {
        view! { cx, <div class="flex justify-start space-x-8">{self.max_jobs} {self.caches}</div> }
            .into_view(cx)
    }
}
