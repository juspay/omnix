use dioxus::prelude::Scope;
use dioxus_signals::{use_signal, Signal};

/// An action to be performed on [AppState]
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Action {
    /// Refresh the [AppState::flake] signal using [AppState::flake_url] signal's current value
    RefreshFlake,

    /// Refresh [AppState::nix_info] signal
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
