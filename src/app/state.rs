//! Application state

use dioxus::prelude::{use_context, Scope};
use dioxus_signals::Signal;

use super::health::get_nix_health;

#[derive(Clone, Copy, Default)]
pub struct AppState {
    pub health_checks: Signal<Option<anyhow::Result<Vec<nix_health::traits::Check>>>>,
}

impl AppState {
    pub async fn update_health_checks(&self) {
        tracing::info!("Updating health checks ...");
        let checks = get_nix_health().await;
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
