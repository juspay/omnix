//! [leptos_query] helpers for working with [server] fns, and useful widgets.
use cfg_if::cfg_if;
use leptos::*;
use leptos_query::*;
use std::{future::Future, hash::Hash};
use tracing::instrument;

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
/// In order to be able to log the name of the server fns, we unfortunately must
/// require them to be 1-ary functions taking tuples, due to a limitation with
/// Rust type system around Fn trait.
///
/// Arguments
/// * `k`: The argument to the server fn
/// * `fetcher`: The server fn to call
#[instrument(
    name = "use_server_query",
    skip(cx, k, fetcher),
    fields(
        fetcher = std::any::type_name::<F>(),
        render_mode=LEPTOS_MODE
    )
)]
pub fn use_server_query<K, V, F, Fu>(
    cx: Scope,
    k: impl Fn() -> K + 'static,
    fetcher: F,
) -> QueryResult<ServerFnResult<V>, impl RefetchFn>
where
    K: Hash + Eq + Clone + std::fmt::Debug + 'static,
    ServerFnResult<V>: Clone + Serializable + 'static,
    Fu: Future<Output = ServerFnResult<V>> + 'static,
    F: Fn(K) -> Fu + 'static,
{
    let span = tracing::Span::current();
    tracing::debug!("use_query");
    leptos_query::use_query(
        cx,
        k,
        move |k| {
            let _enter = span.enter();
            tracing::info!("calling server fn");
            fetcher(k)
        },
        query_options::<ServerFnResult<V>>(),
    )
}

const LEPTOS_MODE: &str = {
    cfg_if! { if #[cfg(feature="ssr")] {
        "ssr"
    } else if #[cfg(feature="hydrate")] {
        "hydrate"
    } else {
        compile_error!("Either ssr or hydrate feature must be enabled");
    }}
};

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
