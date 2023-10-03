mod cli;
mod logging;

use dioxus::prelude::*;
use dioxus_router::prelude::*;

#[tokio::main]
async fn main() {
    // use clap::Parser;
    // let args = crate::cli::Args::parse();

    // FIXME: remove dioxus:/ for release
    dioxus_desktop::launch_cfg(
        App,
        dioxus_desktop::Config::new()
            .with_custom_head(r#"<link rel="stylesheet" href="dioxus://assets/tailwind.css">"#.to_string()),
    )
}

fn App(cx: Scope) -> Element {
    render! {
        div { class: "md:container mx-auto",
            h1 { class: "text-3xl font-bold text-green-400", "nix-browser (Dioxus)" }
            p { "It is WIP!" }
        }
    }
}