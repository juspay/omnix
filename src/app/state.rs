//! Application state

mod datum;

use std::fmt::Display;

use dioxus::prelude::{use_context, use_context_provider, use_future, Scope};
use dioxus_signals::{use_signal, CopyValue, Signal};
use nix_health::NixHealth;
use nix_rs::{
    command::NixCmdError,
    flake::{url::FlakeUrl, Flake},
};
use tokio::task::AbortHandle;

use self::datum::Datum;

/// Our dioxus application state is a struct of [Signal]
///
/// They use [Datum] which is a glorified [Option] to distinguis between initial
/// loading and subsequent refreshing.
///
/// Use [Action] to mutate this state, in addition to [Signal::set].
#[derive(Default, Clone, Copy, Debug, PartialEq)]
pub struct AppState {
    pub nix_info: Signal<Datum<Result<nix_rs::info::NixInfo, SystemError>>>,
    pub health_checks: Signal<Datum<Result<Vec<nix_health::traits::Check>, SystemError>>>,

    pub flake_url: Signal<FlakeUrl>,
    pub flake: Signal<Datum<Result<Flake, NixCmdError>>>,
    pub flake_task_abort: CopyValue<Option<AbortHandle>>,

    pub action: Signal<(usize, Action)>,
}

/// An action to be performed on [AppState]
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Action {
    RefreshFlake,
    #[default]
    GetNixInfo,
}

impl Action {
    /// Return a [Signal] containing only the given [Action]
    ///
    /// The signal value will be the [Action]'s index in the original signal.
    pub fn signal_for<F>(cx: Scope, sig: Signal<(usize, Action)>, f: F) -> Signal<usize>
    where
        F: Fn(Action) -> bool + 'static,
    {
        signal_filter_map(
            cx,
            sig,
            0,
            move |(idx, action)| {
                if f(action) {
                    Some(idx)
                } else {
                    None
                }
            },
        )
    }
}

impl AppState {
    /// Perform an [Action] on the state
    ///
    /// This eventuates an update on the appropriate signals the state holds.
    pub fn act(&self, action: Action) {
        self.action.with_mut(|(i, v)| {
            *i += 1;
            *v = action;
        });
    }

    /// Get the [AppState] from context
    pub fn use_state(cx: Scope) -> Self {
        *use_context(cx).unwrap()
    }

    pub fn provide_state(cx: Scope) {
        tracing::debug!("🏗️ Providing AppState");
        use_context_provider(cx, || {
            tracing::debug!("🔨 Creating AppState default value");
            AppState::default()
        });
        // FIXME: Can we avoid calling build_network multiple times?
        let state = AppState::use_state(cx);
        state.build_network(cx);
    }

    /// Build the Signal network
    ///
    /// If a signal's value is dependent on another signal's value, you must
    /// define that relationship here.
    fn build_network(self, cx: Scope) {
        tracing::debug!("🕸️ Building AppState network");
        // Build `state.flake` signal when `state.flake_url` changes or the
        // RefreshFlake action is triggered
        {
            let flake_url = self.flake_url.read().clone();
            let refresh_action =
                Action::signal_for(cx, self.action, |act| act == Action::RefreshFlake);
            let idx = *refresh_action.read();
            use_future(cx, (&flake_url, &idx), |(flake_url, idx)| async move {
                tracing::info!("Updating flake [{}] {} ...", flake_url, idx);
                // Abort previously running task, otherwise Datum refresh will panic
                // TODO: Refactor this by changing the [Datum] type to be a
                // struct (not enum) containing the
                // `CopyValue<Option<JoinHandle<T>>>`
                self.flake_task_abort.with_mut(|abort_handle| {
                    if let Some(abort_handle) = abort_handle.take() {
                        abort_handle.abort();
                    }
                });
                Datum::refresh_with(self.flake, async move {
                    let join_handle = tokio::spawn(async move {
                        Flake::from_nix(&nix_rs::command::NixCmd::default(), flake_url.clone())
                            .await
                    });
                    *self.flake_task_abort.write() = Some(join_handle.abort_handle());
                    let v = join_handle.await.unwrap();
                    *self.flake_task_abort.write() = None;
                    v
                })
                .await;
            });
        }

        // Build `state.health_checks` when nix_info or nix_env changes
        {
            let nix_info = self.nix_info.read().clone();
            use_future(cx, (&nix_info,), |(nix_info,)| async move {
                if let Some(nix_info) = nix_info.current_value() {
                    Datum::refresh_with(self.health_checks, async {
                        let get_nix_health =
                            move || -> Result<Vec<nix_health::traits::Check>, SystemError> {
                                let nix_info = nix_info
                                    .as_ref()
                                    .map_err(|e| Into::<SystemError>::into(e.to_string()))?;
                                let health_checks = NixHealth::default().run_checks(nix_info, None);
                                Ok(health_checks)
                            };
                        get_nix_health()
                    })
                    .await;
                }
            });
        }

        // Build `state.nix_info` when GetNixInfo action is triggered
        {
            let get_nix_info_action =
                Action::signal_for(cx, self.action, |act| act == Action::GetNixInfo);
            let idx = *get_nix_info_action.read();
            use_future(cx, (&idx,), |(idx,)| async move {
                tracing::info!("Updating nix info [{}] ...", idx);
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
            });
        }
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

/// Like [std::iter::Iterator::filter_map] but applied on a dioxus [Signal]
///
/// Since `Signal`s always have a value, an `initial` value must be provided.
///
/// Upstream issue: https://github.com/DioxusLabs/dioxus/issues/1555
fn signal_filter_map<T, U, F>(cx: Scope, sig: Signal<T>, initial: U, f: F) -> Signal<U>
where
    F: Fn(T) -> Option<U> + 'static,
    T: Copy,
{
    let res: Signal<U> = use_signal(cx, || initial);
    dioxus_signals::use_effect(cx, move || {
        let value = *sig.read();
        if let Some(value) = f(value) {
            res.set(value);
        }
    });
    res
}
