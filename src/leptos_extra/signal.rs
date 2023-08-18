//! [Signal] related helpers for Leptos
use leptos::*;
use tracing::instrument;

/// [provide_context] a new signal of type `T` in the current scope
pub fn provide_signal<T: 'static>(cx: Scope, default: T) {
    let (get, set) = create_signal(cx, default);
    provide_context(cx, (get, set));
}

/// [use_context] the signal of type `T` in the current scope
///
/// If the signal was not provided in a top-level scope (via [provide_signal])
/// this method will panic after tracing an error.
#[instrument(name = "use_signal")]
pub fn use_signal<T>(cx: Scope) -> (ReadSignal<T>, WriteSignal<T>) {
    use_context(cx)
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

/// Map the value inside a nested [Option]-of-[Result]
///
/// This function is efficient in that the inner value is not cloned.
pub fn map_option_result_ref<T1, T2, E>(
    d: &Option<Result<T1, E>>,
    f: impl Fn(&T1) -> T2 + 'static,
) -> Option<Result<T2, E>>
where
    E: Clone,
{
    d.as_ref().map(|r| r.as_ref().map(f).map_err(Clone::clone))
}
