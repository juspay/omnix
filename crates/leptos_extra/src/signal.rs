//! [Signal] related helpers for Leptos
use leptos::*;
use tracing::instrument;

/// [provide_context] a new signal of type `T` in the current scope
pub fn provide_signal<T: 'static>(default: T) {
    let sig = create_rw_signal(default);
    provide_context(sig);
}

/// [use_context] the signal of type `T` in the current scope
///
/// If the signal was not provided in a top-level scope (via [provide_signal])
/// this method will panic after tracing an error.
#[instrument(name = "use_signal")]
pub fn use_signal<T>() -> RwSignal<T> {
    use_context()
        .ok_or_else(|| {
            // This happens if the dev forgets to call `provide_signal::<T>` in
            // the parent scope
            let msg = format!(
                "no signal provided for type: {}",
                std::any::type_name::<T>()
            );
            tracing::error!(msg);
            msg
        })
        .unwrap()
}

/// Extends [SignalWith] to add a `with_result` method that operates on the
/// inner value, avoiding the need to clone it.
pub trait SignalWithResult<T, E>: SignalWith<Value = Option<Result<T, E>>> {
    /// Like [SignalWith::with] but operates on the inner [Result] value without cloning it.
    fn with_result<U>(&self, f: impl Fn(&T) -> U + 'static) -> Option<Result<U, E>>
    where
        E: Clone,
    {
        self.with(move |d| d.map_option_result(f))
    }
}

impl<T, E> SignalWithResult<T, E> for Signal<Option<Result<T, E>>> {}

/// Functions unique to [Option] of [Result] values
pub trait OptionResult<T, E> {
    /// Map the value inside a nested [Option]-of-[Result]
    ///
    /// This function is efficient in that the inner value is not cloned.
    fn map_option_result<U>(&self, f: impl Fn(&T) -> U + 'static) -> Option<Result<U, E>>
    where
        E: Clone;

    /// Like [[Option::unwrap_or]] but unwraps the nested value
    fn unwrap_option_result_value_or(&self, default: T) -> T
    where
        T: Clone;
}

impl<T, E> OptionResult<T, E> for Option<Result<T, E>> {
    fn map_option_result<U>(&self, f: impl Fn(&T) -> U + 'static) -> Option<Result<U, E>>
    where
        E: Clone,
    {
        self.as_ref()
            .map(|r| r.as_ref().map(f).map_err(Clone::clone))
    }

    fn unwrap_option_result_value_or(&self, default: T) -> T
    where
        T: Clone,
    {
        self.as_ref()
            .and_then(|r| r.as_ref().ok())
            .cloned()
            .unwrap_or(default)
    }
}
