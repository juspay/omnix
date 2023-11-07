use std::{fmt::Display, future::Future};

use dioxus::prelude::*;
use dioxus_signals::{CopyValue, Signal};
use tokio::task::AbortHandle;

/// Represent loading/refreshing state of UI data
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Datum<T> {
    /// The current value of the datum
    value: Option<T>,
    /// If the datum is currently being loaded or refresh, this contains the
    /// [AbortHandle] to abort that loading/refreshing process.
    task: CopyValue<Option<AbortHandle>>,
}

impl<T> Default for Datum<T> {
    fn default() -> Self {
        Self {
            value: None,
            task: CopyValue::default(),
        }
    }
}

impl<T> Datum<T> {
    pub fn is_loading_or_refreshing(&self) -> bool {
        self.task.read().is_some()
    }

    /// Get the inner value if available
    pub fn current_value(&self) -> Option<&T> {
        self.value.as_ref()
    }

    /// Refresh the datum [Signal] using the given function
    ///
    /// If a previous refresh is still running, it will be cancelled.
    pub async fn refresh_with<F>(signal: Signal<Datum<T>>, f: F)
    where
        F: Future<Output = T> + Send + 'static,
        T: Send + 'static,
    {
        // Cancel existing fetcher if any.
        signal.with_mut(move |x| {
            if let Some(abort_handle) = x.task.take() {
                tracing::warn!(
                    "🍒 Cancelling previous refresh task for {}",
                    std::any::type_name::<T>()
                );
                abort_handle.abort();
            }
        });

        // NOTE: We must spawn a tasks (using tokio::spawn), otherwise this
        // will run in main desktop thread, and will hang at some point.
        let join_handle = tokio::spawn(f);

        // Store the [AbortHandle] for cancelling latter.
        let abort_handle = join_handle.abort_handle();
        signal.with_mut(move |x| {
            *x.task.write() = Some(abort_handle);
        });

        // Wait for result and update the signal state.
        match join_handle.await {
            Ok(val) => {
                signal.with_mut(move |x| {
                    tracing::debug!("🍒 Setting {} datum value", std::any::type_name::<T>());
                    x.value = Some(val);
                    *x.task.write() = None;
                });
            }
            Err(err) => {
                if !err.is_cancelled() {
                    tracing::error!("🍒 Datum refresh failed: {err}");
                    signal.with_mut(move |x| {
                        *x.task.write() = None;
                    });
                }
                // x.task will be set to None by the caller who cancelled us, so
                // we need not do anything here.
            }
        }
    }
}

impl<T, E: Display> Datum<Result<T, E>> {
    /// Render the result datum with the given component
    ///
    /// The error message will be rendered appropriately. If the datum is
    /// unavailable, nothing will be rendered (loading state is rendered
    /// differently)
    pub fn render_with<'a, F>(&self, cx: &'a Scoped<'a, ()>, component: F) -> Element<'a>
    where
        F: FnOnce(&T) -> Element<'a>,
    {
        match self.current_value()? {
            Ok(value) => component(value),
            Err(err) => render! {
                div {
                    class: "p-4 my-1 text-left text-sm font-mono text-white bg-red-500 rounded",
                    "Error: {err}"
                }
            },
        }
    }
}
