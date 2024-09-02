//! Application state

mod datum;
mod db;
mod error;
mod refresh;

use dioxus::prelude::*;
use dioxus_signals::{Readable, Signal, Writable};
use nix_health::NixHealth;
use nix_rs::{
    config::NixConfig,
    flake::{url::FlakeUrl, Flake},
    info::NixInfo,
    version::NixVersion,
};

use self::{datum::Datum, error::SystemError, refresh::Refresh};

/// Our dioxus application state is a struct of [Signal]s that store app state.
///
/// They use [Datum] which is a glorified [Option] to distinguish between initial
/// loading and subsequent refreshing.
#[derive(Default, Clone, Copy, Debug, PartialEq)]
pub struct AppState {
    /// [NixInfo] as detected on the user's system
    pub nix_info: Signal<Datum<Result<NixInfo, SystemError>>>,
    pub nix_info_refresh: Signal<Refresh>,

    /// User's Nix health [nix_health::traits::Check]s
    pub health_checks: Signal<Datum<Result<Vec<nix_health::traits::Check>, SystemError>>>,
    pub health_checks_refresh: Signal<Refresh>,

    /// User selected [FlakeUrl]
    pub flake_url: Signal<Option<FlakeUrl>>,
    /// Trigger to refresh [AppState::flake]
    pub flake_refresh: Signal<Refresh>,
    /// [Flake] for [AppState::flake_url]
    pub flake: Signal<Datum<Result<Flake, SystemError>>>,

    /// Cached [Flake] values indexed by [FlakeUrl]
    ///
    /// Most recently updated flakes appear first.
    pub flake_cache: Signal<db::FlakeCache>,
}

impl AppState {
    fn new() -> Self {
        tracing::info!("🔨 Creating new AppState");
        // TODO: Should we use new_synced_storage, instead? To allow multiple app windows?
        let flake_cache = db::FlakeCache::new_signal();
        AppState {
            flake_cache,
            ..AppState::default()
        }
    }

    /// Get the [AppState] from context
    pub fn use_state() -> Self {
        use_context::<Self>()
    }

    pub fn provide_state() {
        tracing::debug!("🏗️ Providing AppState");
        let mut state = use_context_provider(Self::new);
        // FIXME: Can we avoid calling build_network multiple times?
        state.build_network();
    }

    /// Return the [String] representation of the current [AppState::flake_url] value. If there is none, return empty string.
    pub fn get_flake_url_string(&self) -> String {
        self.flake_url
            .read()
            .clone()
            .map_or("".to_string(), |url| url.to_string())
    }

    pub fn set_flake_url(&mut self, url: FlakeUrl) {
        tracing::info!("setting flake url to {}", &url);
        self.flake_url.set(Some(url));
    }

    /// Empty flake related data (`flake_url` and `flake`)
    pub fn reset_flake_data(&mut self) {
        tracing::info!("empty flake data");
        self.flake.set(Datum::default());
        self.flake_url.set(None);
    }
}

impl AppState {
    /// Build the Signal network
    ///
    /// If a signal's value is dependent on another signal's value, you must
    /// define that relationship here.
    fn build_network(&mut self) {
        tracing::debug!("🕸️ Building AppState network");
        // Build `state.flake` signal dependent signals change
        {
            // ... when [AppState::flake_url] changes.
            let flake_url = self.flake_url;
            let flake_cache = self.flake_cache;
            let mut flake_refresh = self.flake_refresh;
            let mut flake = self.flake;
            let _ = use_resource(move || async move {
                if let Some(flake_url) = flake_url.read().clone() {
                    let maybe_flake = flake_cache.read().get(&flake_url);
                    if let Some(cached_flake) = maybe_flake {
                        Datum::set_value(&mut flake, Ok(cached_flake)).await;
                    } else {
                        flake_refresh.write().request_refresh();
                    }
                }
            });
            // ... when refresh button is clicked.
            let flake_refresh = self.flake_refresh;
            let flake_url = self.flake_url;
            let mut flake = self.flake;
            let mut flake_cache = self.flake_cache;
            let _ = use_resource(move || async move {
                let nixcmd = nix_rs::command::NixCmd::get().await;
                let flake_url = flake_url.read().clone();
                let refresh = *flake_refresh.read();
                if let Some(flake_url) = flake_url {
                    let flake_url_2 = flake_url.clone();
                    tracing::info!("Updating flake [{}] refresh={} ...", &flake_url, refresh);
                    let res = Datum::refresh_with(&mut flake, async move {
                        let nix_version = NixVersion::from_nix(nixcmd)
                            .await
                            .map_err(|e| Into::<SystemError>::into(e.to_string()))?;
                        let nix_config = NixConfig::from_nix(nixcmd, &nix_version)
                            .await
                            .map_err(|e| Into::<SystemError>::into(e.to_string()))?;
                        Flake::from_nix(nixcmd, &nix_config, flake_url_2)
                            .await
                            .map_err(|e| Into::<SystemError>::into(e.to_string()))
                    })
                    .await;
                    if let Some(Ok(flake)) = res {
                        flake_cache.with_mut(|cache| {
                            cache.update(flake_url, flake);
                        });
                    }
                }
            });
        }

        // Build `state.health_checks`
        {
            let nix_info = self.nix_info;
            let health_checks_refresh = self.health_checks_refresh;
            let mut health_checks = self.health_checks;
            let _ = use_resource(move || async move {
                let nix_info = nix_info.read().clone();
                let refresh = *health_checks_refresh.read();
                if let Some(nix_info) = nix_info.current_value().map(|x| {
                    x.as_ref()
                        .map_err(|e| Into::<SystemError>::into(e.to_string()))
                        .cloned()
                }) {
                    tracing::info!("Updating nix health [{}] ...", refresh);
                    Datum::refresh_with(&mut health_checks, async move {
                        let health_checks = NixHealth::default().run_checks(&nix_info?, None);
                        Ok(health_checks)
                    })
                    .await;
                }
            });
        }

        // Build `state.nix_info`
        {
            let mut nix_info = self.nix_info;
            let nix_info_refresh = self.nix_info_refresh;
            let _ = use_resource(move || async move {
                let refresh = *nix_info_refresh.read();
                tracing::info!("Updating nix info [{}] ...", refresh);
                Datum::refresh_with(&mut nix_info, async {
                    let ver = NixVersion::get()
                        .await
                        .as_ref()
                        .map_err(|e| Into::<SystemError>::into(e.to_string()))?;
                    let cfg = NixConfig::get()
                        .await
                        .as_ref()
                        .map_err(|e| Into::<SystemError>::into(e.to_string()))?;
                    NixInfo::new(*ver, cfg.clone())
                        .await
                        .map_err(|e| SystemError {
                            message: format!("Error getting nix info: {:?}", e),
                        })
                })
                .await;
            });
        }
    }
}
