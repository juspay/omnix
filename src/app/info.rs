//! Frontend UI entry point

use leptos::*;
use leptos_meta::*;
use nix_rs::info::NixInfo;

use crate::widget::*;
use leptos_extra::query::{self, RefetchQueryButton};

/// Nix information
#[component]
pub fn NixInfoRoute(cx: Scope) -> impl IntoView {
    let title = "Nix Info";
    let result = query::use_server_query(cx, || (), get_nix_info);
    let data = result.data;
    view! { cx,
        <Title text=title/>
        <h1 class="text-5xl font-bold">{title}</h1>
        <RefetchQueryButton result query=|| ()/>
        <div class="my-1 text-left">
            <SuspenseWithErrorHandling>{data}</SuspenseWithErrorHandling>
        </div>
    }
}

/// Determine [NixInfo] on the user's system
#[server(GetNixInfo, "/api")]
pub async fn get_nix_info(_unit: ()) -> Result<NixInfo, ServerFnError> {
    let v = NixInfo::from_nix(&nix_rs::command::NixCmd::default()).await?;
    Ok(v)
}
