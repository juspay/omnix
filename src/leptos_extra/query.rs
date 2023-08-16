//! [leptos_query] helpers for working with [server] fns, and useful widgets.
use core::pin::Pin;
use leptos::server_fn::ServerFn;
use leptos::*;
use leptos_query::*;
use std::{fmt::Display, future::Future, hash::Hash, str::FromStr};
use tracing::{info_span, instrument};

/// The result type of Leptos [server] function returning a `T`
pub type ServerFnResult<T> = Result<T, ServerFnError>;

/// Sensible [QueryOptions] defaults for an app
pub fn query_options<V>() -> QueryOptions<V> {
    QueryOptions {
        // Disable staleness so the query is not refetched on every route switch.
        stale_time: None,
        ..Default::default()
    }
}

/// Like [use_query] but specifically meant for server functions, does logging
/// via [tracing] and uses [query_options] always.
///
/// Arguments
/// * `k`: The argument to the server fn
/// * `mk_args`: How to convert `k` into the server fn's argument type
#[instrument(name = "use_server_query", skip(mk_args, k))]
pub fn use_server_query<S, K>(
    cx: Scope,
    k: impl Fn() -> K + 'static,
    mk_args: impl Fn(K) -> S + 'static,
) -> QueryResult<ServerFnResult<<S as ServerFn<Scope>>::Output>, impl RefetchFn>
where
    K: Hash + Eq + Clone + std::fmt::Debug + 'static,
    ServerFnResult<<S as ServerFn<Scope>>::Output>: Clone + Serializable + 'static,
    S: Clone + std::fmt::Debug + ServerFn<leptos::Scope>,
{
    let span = tracing::Span::current();
    tracing::info!("use_query");
    leptos_query::use_query(
        cx,
        k,
        move |k| {
            let _enter = span.enter();
            let args: S = mk_args(k);
            call_server_fn::<S>(cx, &args)
        },
        query_options::<ServerFnResult<<S as ServerFn<Scope>>::Output>>(),
    )
}

/// Manually call a [server] fn given its arguments
#[instrument(name = "call_server_fn")]
fn call_server_fn<S>(
    cx: Scope,
    args: &S,
) -> Pin<Box<dyn Future<Output = Result<S::Output, ServerFnError>>>>
where
    S: Clone + std::fmt::Debug + ServerFn<leptos::Scope>,
{
    #[cfg(feature = "ssr")]
    let v = {
        let span = info_span!("ssr");
        let _enter = span.enter();
        tracing::info!(type_ = std::any::type_name::<S>());
        S::call_fn(args.clone(), cx)
    };
    #[cfg(not(feature = "ssr"))]
    let v = {
        let span = info_span!("hydrate");
        let _enter = span.enter();
        tracing::info!(type_ = std::any::type_name::<S>());
        S::call_fn_client(args.clone(), cx)
    };
    v
}

/// Input element component to pass arguments to a [leptos_query] query
///
/// A label, input element, and datalist are rendered, as well as error div.
/// [FromStr::from_str] is used to parse the input value into `K`.
///
/// Arguments:
/// * `id`: The id of the input element
/// * `suggestions`: The initial suggestions to show in the datalist
/// * `query`: Input element value is initialized with this [ReadSignal]
/// * `set_query`: Input element will set this [WriteSignal]
#[component]
pub fn QueryInput<K>(
    cx: Scope,
    id: &'static str,
    /// Initial suggestions to show in the datalist
    suggestions: Vec<K>,
    query: ReadSignal<K>,
    set_query: WriteSignal<K>,
) -> impl IntoView
where
    K: ToString + FromStr + Hash + Eq + Clone + Display + 'static,
    <K as std::str::FromStr>::Err: Display,
{
    let datalist_id = &format!("{}-datalist", id);
    // Input query to the server fn
    // Errors in input element (based on [FromStr::from_str])
    let (input_err, set_input_err) = create_signal(cx, None::<String>);
    view! { cx,
        <label for=id>"Load a Nix flake"</label>
        <input
            list=datalist_id
            id=id.to_string()
            type="text"
            class="w-full p-1 font-mono"
            on:change=move |ev| {
                match FromStr::from_str(&event_target_value(&ev)) {
                    Ok(url) => {
                        set_query(url);
                        set_input_err(None)
                    }
                    Err(e) => set_input_err(Some(e.to_string())),
                }
            }

            prop:value=move || query().to_string()
        />
        <span class="text-red-500">{input_err}</span>
        // TODO: use local storage, and cache user's inputs
        <datalist id=datalist_id>
            {suggestions
                .iter()
                .map(|s| view! { cx, <option value=s.to_string()></option> })
                .collect_view(cx)}

        </datalist>
    }
}

/// Button component to refresh the given [leptos_query] query.
///
/// Arguments
/// * `result`: The query result to refresh
/// * `query`: The value to pass to [invalidate_query]
#[component]
pub fn RefetchQueryButton<K, V, R, F>(
    cx: Scope,
    result: QueryResult<ServerFnResult<V>, R>,
    query: F,
) -> impl IntoView
where
    K: Hash + Eq + Clone + std::fmt::Debug + 'static,
    ServerFnResult<V>: Clone + Serializable + 'static,
    R: RefetchFn,
    F: Fn() -> K + 'static,
{
    view! { cx,
        <button
            class="p-1 text-white shadow border-1 bg-primary-700 disabled:bg-base-400 disabled:text-black"
            disabled=move || result.is_fetching.get()
            on:click=move |_| {
                let k = query();
                tracing::debug!("Invalidating query");
                use_query_client(cx).invalidate_query::<K, ServerFnResult<V>>(k);
            }
        >

            {move || if result.is_fetching.get() { "Fetching..." } else { "Re-fetch" }}
        </button>
    }
}
