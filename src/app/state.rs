//! Application state

use dioxus::prelude::{use_context, use_context_provider, use_future, Scope};
use dioxus_signals::Signal;
use nix_rs::command::NixCmdError;

use super::health::{get_nix_health, NixHealthError};

#[derive(Default, Clone, Copy)]
pub struct AppState {
    pub nix_info: Signal<Option<Result<nix_rs::info::NixInfo, NixCmdError>>>,
    // pub nix_env: Signal<Option<Result<nix_rs::env::NixEnv>>>,
    pub health_checks: Signal<Option<Result<Vec<nix_health::traits::Check>, NixHealthError>>>,
}

impl AppState {
    pub async fn initialize(&self) {
        tracing::info!("Initializing app state");
        if self.nix_info.read().is_none() {
            self.update_nix_info().await;
        }
        if self.health_checks.read().is_none() {
            self.update_health_checks().await;
        }
    }

    pub async fn update_nix_info(&self) {
        tracing::info!("Updating nix info ...");
        let nix_info = nix_rs::info::NixInfo::from_nix(&nix_rs::command::NixCmd::default()).await;
        tracing::info!("Got nix info, about to mut");
        self.nix_info.with_mut(move |x| {
            *x = Some(nix_info);
            tracing::info!("Updated nix info");
        });
    }

    pub async fn update_health_checks(&self) {
        // TODO: reuse nix_info
        // self.update_nix_info().await;
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

    pub fn provide_state(cx: Scope) {
        use_context_provider(cx, AppState::default);
        let state = AppState::use_state(cx);
        use_future(cx, (), |_| async move {
            state.initialize().await;
        });
    }
}
