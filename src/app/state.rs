//! Application state

use anyhow::Result;
use dioxus::prelude::{use_context, Scope};
use dioxus_signals::Signal;

use super::health::get_nix_health;

#[derive(Clone, Copy, Default)]
pub struct AppState {
    // pub nix_info: Signal<Option<Result<nix_rs::info::NixInfo>>>,
    // pub nix_env: Signal<Option<Result<nix_rs::env::NixEnv>>>,
    pub health_checks: Signal<Option<Result<Vec<nix_health::traits::Check>>>>,
}

impl AppState {
    pub async fn initialize(&self) {
        tracing::info!("Initializing app state");
        if self.health_checks.read().is_none() {
            self.update_health_checks().await;
        }
    }
    pub async fn update_health_checks(&self) {
        tracing::info!("Updating health checks ...");
        let checks = get_nix_health().await;
        tracing::info!("Got health checks, about to mut");
        self.health_checks.with_mut(move |x| {
            *x = Some(checks);
            tracing::info!("Updated health checks");
        });
    }

    /// Get the [AppState] from context
    pub fn use_state(cx: Scope) -> Self {
        *use_context(cx).unwrap()
    }
}
