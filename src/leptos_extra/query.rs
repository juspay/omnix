//! [leptos_query] helpers
///
/// This module could be simplified after
/// https://github.com/nicoburniske/leptos_query/issues/7
use leptos::*;
use leptos_query::*;
use server_fn::ServerFn;
use std::{fmt::Display, future::Future, hash::Hash, marker::PhantomData, pin::Pin, str::FromStr};
use tracing::{info_span, instrument};

use crate::leptos_extra::signal::use_signal;

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
#[instrument(name = "use_server_query", skip(k))]
pub fn use_server_query<S>(
    cx: Scope,
    k: impl Fn() -> S + 'static,
) -> ServerQueryResult<<S as ServerFn<Scope>>::Output, impl RefetchFn>
where
    S: Hash + Eq + Clone + ServerFn<Scope> + std::fmt::Debug + 'static,
    ServerQueryVal<<S as ServerFn<Scope>>::Output>: Clone + Serializable + 'static,
{
    tracing::info!(type_ = std::any::type_name::<S>());
    leptos_query::use_query(
        cx,
        k,
        move |k| call_server_fn::<S>(cx, &k),
        query_options::<ServerQueryVal<<S as ServerFn<Scope>>::Output>>(),
    )
}

#[instrument(name = "call_server_fn")]
pub fn call_server_fn<S>(
    cx: Scope,
    args: &S,
) -> Pin<Box<dyn Future<Output = Result<S::Output, ServerFnError>>>>
where
    S: Clone + std::fmt::Debug + ServerFn<Scope>,
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

/// An input element tied to a [leptos_use::Query]
#[component]
pub fn ServerQueryInput<S>(
    cx: Scope,
    /// Initial suggestions to show in the datalist
    suggestions: Vec<S>,
    #[allow(unused_variables)] serverfn: std::marker::PhantomData<S>,
) -> impl IntoView
where
    S: ToString + FromStr + Hash + Eq + Clone + leptos::server_fn::ServerFn<leptos::Scope>,
    <S as std::str::FromStr>::Err: Display,
    ServerQueryVal<<S as leptos::server_fn::ServerFn<leptos::Scope>>::Output>:
        Clone + Serializable + 'static,
{
    let id = &format!("{}-input", std::any::type_name::<S>());
    let datalistId = &format!("{}-datalist", std::any::type_name::<S>());
    // Input query to the server fn
    // TODO: If we are using use_signal, we might as well abstract this out at higher level
    let (query, set_query) = use_signal::<S>(cx);
    // Errors in input element (based on [FromStr::from_str])
    let (input_err, set_input_err) = create_signal(cx, None::<String>);
    view! { cx,
        <label for=id>"Load a Nix flake"</label>
        <input
            list=datalistId
            id=id
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
        <datalist id=datalistId>
            {suggestions
                .iter()
                .map(|s| view! { cx, <option value=s.to_string()></option> })
                .collect_view(cx)}

        </datalist>
    }
}

/// Button to refresh the given [leptos_query] query.
/// TODO: Use this, by implement traits for other server functions
#[component]
pub fn RefetchQueryButton<S, R>(
    cx: Scope,
    res: ServerQueryResult<<S as ServerFn<Scope>>::Output, R>,
    k: S,
    #[allow(unused_variables)] serverfn: PhantomData<S>,
) -> impl IntoView
where
    S: Hash + Eq + Clone + ServerFn<Scope> + std::fmt::Debug + 'static,
    ServerQueryVal<<S as ServerFn<Scope>>::Output>: Clone + Serializable + 'static,
    R: RefetchFn,
{
    view! { cx,
        <button
            class="p-1 text-white shadow border-1 bg-primary-700 disabled:bg-base-400 disabled:text-black"
            disabled=move || res.is_fetching.get()
            on:click=move |_| {
                tracing::debug!("Invalidating query");
                use_query_client(cx)
                    .invalidate_query::<
                        S,
                        ServerQueryVal<<S as ServerFn<Scope>>::Output>,
                    >(k.clone());
            }
        >

            {move || if res.is_fetching.get() { "Fetching..." } else { "Re-fetch" }}
        </button>
    }
}
