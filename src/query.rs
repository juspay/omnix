//! [leptos_query] queries for our app
///
/// This module could be simplified after
/// https://github.com/nicoburniske/leptos_query/issues/7
use leptos::*;
use leptos_query::*;
use server_fn::ServerFn;
use std::{future::Future, hash::Hash, pin::Pin};

use crate::nix::{
    health::{get_nix_health, NixHealth},
    info::{get_nix_info, NixInfo},
};

/// Type alias for [QueryResult] specialized for Leptos [server] functions
type ServerQueryResult<T, R> = QueryResult<ServerQueryVal<T>, R>;
pub type ServerQueryVal<T> = Result<T, ServerFnError>;

fn query_options<V>() -> QueryOptions<V> {
    QueryOptions {
        // Disable staleness so the query is not refetched on every route switch.
        stale_time: None,
        ..Default::default()
    }
}

/// Like [use_query] gut for server functions
pub fn use_server_query<S>(
    cx: Scope,
    k: impl Fn() -> S + 'static,
) -> ServerQueryResult<<S as ServerFn<Scope>>::Output, impl RefetchFn>
where
    S: Hash + Eq + Clone + ServerFn<Scope> + 'static,
    ServerQueryVal<<S as ServerFn<Scope>>::Output>: Clone + Serializable + 'static,
{
    tracing::info!("use_server_query");
    leptos_query::use_query(
        cx,
        k,
        move |k| call_server_fn::<S>(cx, &k),
        query_options::<ServerQueryVal<<S as ServerFn<Scope>>::Output>>(),
    )
}

pub fn call_server_fn<S>(
    cx: Scope,
    args: &S,
) -> Pin<Box<dyn Future<Output = Result<S::Output, ServerFnError>>>>
where
    S: Clone + ServerFn<Scope>,
{
    tracing::info!("call_server_fn");
    #[cfg(feature = "ssr")]
    let v = S::call_fn(args.clone(), cx);
    #[cfg(not(feature = "ssr"))]
    let v = S::call_fn_client(args.clone(), cx);
    v
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
///
/// TODO: Change this to work at server fn level
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
