//! Nix health check UI

use leptos::*;
use leptos_meta::*;
use tracing::instrument;

use crate::widget::*;
use leptos_extra::query::{self, RefetchQueryButton};

/// Nix health checks
#[component]
pub fn NixHealthRoute(cx: Scope) -> impl IntoView {
    let title = "Nix Health";
    let result = query::use_server_query(cx, || (), get_nix_health);
    let data = result.data;
    view! { cx,
        <Title text=title/>
        <h1 class="text-5xl font-bold">{title}</h1>
        <RefetchQueryButton result query=|| ()/>
        <div class="my-1">
            <SuspenseWithErrorHandling>{data}</SuspenseWithErrorHandling>
        </div>
    }
}

/// Get [NixHealth] information
#[instrument(name = "nix-health")]
#[server(GetNixHealth, "/api")]
pub async fn get_nix_health(_unit: ()) -> Result<nix_rs::health::NixHealth, ServerFnError> {
    use nix_rs::health::{traits::Check, NixHealth};
    use nix_rs::info;
    let info = info::NixInfo::from_nix(&nix_rs::command::NixCmd::default()).await?;
    Ok(NixHealth::check(&info))
}
