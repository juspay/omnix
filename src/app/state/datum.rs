use std::{fmt::Display, future::Future};

use dioxus::prelude::*;
use dioxus_signals::{CopyValue, Signal};
use tokio::task::{AbortHandle, JoinHandle};

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
    pub fn current_value<'a>(&'a self) -> Option<&'a T> {
        self.value.as_ref()
    }

    /// Set the datum value
    ///
    /// Use [refresh_with] if the value is produced by a long-running task.
    fn set_value(&mut self, value: T) {
        tracing::debug!("🍒 Setting {} datum value", std::any::type_name::<T>());
        self.value = Some(value);
        // TODO: task state?
    }

    /// Refresh the datum [Signal] using the given function
    ///
    /// Refresh state is automatically set.
    pub async fn refresh_with<F>(signal: Signal<Self>, f: F)
    where
        F: Future<Output = JoinHandle<T>>,
    {
        signal.with_mut(move |x| {
            if let Some(abort_handle) = x.task.take() {
                abort_handle.abort();
            }
        });
        let join_handle = f.await;
        let abort_handle = join_handle.abort_handle();
        signal.with_mut(move |x| {
            *x.task.write() = Some(abort_handle);
        });
        // TODO: handle abort events
        let val = join_handle.await.unwrap();
        signal.with_mut(move |x| {
            x.set_value(val);
            *x.task.write() = None;
        });
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
