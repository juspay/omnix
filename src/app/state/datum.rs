use std::{fmt::Display, future::Future};

use dioxus::prelude::*;
use dioxus_signals::Signal;

/// Represent loading/refreshing state of UI data
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Datum<T> {
    #[default]
    Loading,
    Available {
        value: T,
        refreshing: bool,
    },
}

impl<T> Datum<T> {
    pub fn is_loading_or_refreshing(&self) -> bool {
        matches!(
            self,
            Datum::Loading
                | Datum::Available {
                    value: _,
                    refreshing: true
                }
        )
    }

    /// Get the inner value if available
    pub fn current_value(&self) -> Option<&T> {
        match self {
            Datum::Loading => None,
            Datum::Available {
                value: x,
                refreshing: _,
            } => Some(x),
        }
    }

    pub fn set_value(&mut self, value: T) {
        tracing::info!("🍒 Setting {} datum value", std::any::type_name::<T>());
        *self = Datum::Available {
            value,
            refreshing: false,
        }
    }

    pub fn mark_refreshing(&mut self) {
        if let Datum::Available {
            value: _,
            refreshing,
        } = self
        {
            if *refreshing {
                panic!("Cannot refresh already refreshing data");
            }
            tracing::info!(
                "🍒 Marking {} datum as refreshing",
                std::any::type_name::<T>()
            );
            *refreshing = true;
        }
    }

    /// Refresh the datum [Signal] using the given function
    ///
    /// Refresh state is automatically set.
    pub async fn refresh_with<F>(signal: Signal<Self>, f: F)
    where
        F: Future<Output = T>,
    {
        signal.with_mut(move |x| {
            x.mark_refreshing();
        });
        let val = f.await;
        signal.with_mut(move |x| {
            x.set_value(val);
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
