//! [leptos_query] queries for our app
///
/// This module could be simplified after
/// https://github.com/nicoburniske/leptos_query/issues/7
use leptos::*;
use leptos_query::*;
use std::hash::Hash;

use crate::nix::{
    health::{get_nix_health, NixHealth},
    info::{get_nix_info, NixInfo},
};

/// Type alias for [QueryResult] specialized for Leptos [server] functions
type ServerQueryResult<T, R> = QueryResult<Result<T, ServerFnError>, R>;

fn query_options<V>() -> QueryOptions<V> {
    QueryOptions {
        // Disable staleness so the query is not refetched on every route switch.
        stale_time: None,
        ..Default::default()
    }
}

/// Query [get_nix_info]
pub fn use_nix_info_query(cx: Scope) -> ServerQueryResult<NixInfo, impl RefetchFn> {
    leptos_query::use_query(
        cx,
        || (),
        |()| async move { get_nix_info().await },
        query_options(),
    )
}

/// Query [get_nix_health]
pub fn use_nix_health_query(cx: Scope) -> ServerQueryResult<NixHealth, impl RefetchFn> {
    leptos_query::use_query(
        cx,
        || (),
        |()| async move { get_nix_health().await },
        query_options(),
    )
}

/// Button to refresh the given [leptos_query] query.
#[component]
pub fn RefetchQueryButton<K, V, R>(cx: Scope, res: QueryResult<V, R>, k: K) -> impl IntoView
where
    K: Hash + Eq + Clone + 'static,
    V: Clone + Serializable + 'static,
    R: RefetchFn,
{
    view! { cx,
        <button
            class="p-1 text-white shadow border-1 bg-primary-700 disabled:bg-base-400 disabled:text-black"
            disabled=move || res.is_fetching.get()
            on:click=move |_| {
                tracing::debug!("Invalidating query");
                use_query_client(cx).invalidate_query::<K, V>(k.clone());
            }
        >

            {move || if res.is_fetching.get() { "Fetching..." } else { "Re-fetch" }}
        </button>
    }
}
