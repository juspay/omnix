use std::future::Future;

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
