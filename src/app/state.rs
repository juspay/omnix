//! Application state

mod datum;

use std::fmt::Display;

use dioxus::prelude::{use_context, use_context_provider, use_future, Scope};
use dioxus_signals::Signal;
use nix_health::NixHealth;
use nix_rs::{
    command::NixCmdError,
    flake::{url::FlakeUrl, Flake},
};
use tracing::instrument;

use self::datum::Datum;

/// Our dioxus application state is a struct of [Signal]
///
/// They use [Datum] which is a glorified [Option] to distinguis between initial
/// loading and subsequent refreshing.
#[derive(Default, Clone, Copy, Debug)]
pub struct AppState {
    pub nix_info: Signal<Datum<Result<nix_rs::info::NixInfo, SystemError>>>,
    pub nix_env: Signal<Datum<Result<nix_rs::env::NixEnv, SystemError>>>,
    pub health_checks: Signal<Datum<Result<Vec<nix_health::traits::Check>, SystemError>>>,

    pub flake_url: Signal<FlakeUrl>,
    pub flake: Signal<Datum<Result<Flake, NixCmdError>>>,
}

impl AppState {
    pub async fn initialize(&self) {
        tracing::info!("Initializing app state");
        // Initializing health checks automatially initializes other signals.
        self.update_health_checks().await;
    }

    #[instrument(name = "update-nix-info", skip(self))]
    pub async fn update_nix_info(&self) {
        tracing::debug!("Updating nix info ...");
        Datum::refresh_with(self.nix_info, async {
            // NOTE: Without tokio::spawn, this will run in main desktop thread,
            // and will hang at some point.
            let nix_info = tokio::spawn(async move {
                nix_rs::info::NixInfo::from_nix(&nix_rs::command::NixCmd::default())
                    .await
                    .map_err(|e| SystemError {
                        message: format!("Error getting nix info: {:?}", e),
                    })
            })
            .await
            .unwrap();
            tracing::debug!("Got nix info, about to mut");
            nix_info
        })
        .await;
    }

    #[instrument(name = "update-nix-env", skip(self))]
    pub async fn update_nix_env(&self) {
        tracing::debug!("Updating nix env ...");
        Datum::refresh_with(self.nix_env, async {
            let nix_env = tokio::spawn(async move {
                nix_rs::env::NixEnv::detect(None)
                    .await
                    .map_err(|e| e.to_string().into())
            })
            .await
            .unwrap();
            tracing::debug!("Got nix env, about to mut");
            nix_env
        })
        .await;
    }

    #[instrument(name = "update-health-checks", skip(self))]
    pub async fn update_health_checks(&self) {
        tracing::debug!("Updating health checks ...");
        Datum::refresh_with(self.health_checks, async {
            // Update depenencies
            self.update_nix_info().await;
            self.update_nix_env().await;
            let get_nix_health = move || -> Result<Vec<nix_health::traits::Check>, SystemError> {
                let nix_env = self.nix_env.read();
                let nix_env: &nix_rs::env::NixEnv = nix_env
                    .current_value()
                    .unwrap()
                    .as_ref()
                    .map_err(|e| Into::<SystemError>::into(e.to_string()))?;
                let nix_info = self.nix_info.read();
                let nix_info: &nix_rs::info::NixInfo =
                    nix_info
                        .current_value()
                        .unwrap()
                        .as_ref()
                        .map_err(|e| Into::<SystemError>::into(e.to_string()))?;
                let health_checks = NixHealth::default().run_checks(nix_info, nix_env);
                Ok(health_checks)
            };
            let health_checks = get_nix_health();
            tracing::debug!("Got health checks, about to mut");
            health_checks
        })
        .await;
    }

    #[instrument(name = "set-flake-url", skip(self))]
    pub async fn set_flake_url(&self, url: &FlakeUrl) {
        // TODO: Can we use derived signals here?
        self.flake_url.set(url.clone());
        self.update_flake().await;
    }

    #[instrument(name = "update-flake", skip(self))]
    pub async fn update_flake(&self) {
        tracing::debug!("Updating flake ...");
        Datum::refresh_with(self.flake, async {
            let flake_url = self.flake_url.read().clone();
            let flake = tokio::spawn(async move {
                Flake::from_nix(&nix_rs::command::NixCmd::default(), flake_url.clone()).await
            })
            .await
            .unwrap();
            tracing::debug!("Got flake, about to mut");
            flake
        })
        .await;
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

/// Catch all error to use in UI components
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SystemError {
    pub message: String,
}

impl Display for SystemError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

impl From<String> for SystemError {
    fn from(message: String) -> Self {
        Self { message }
    }
}
