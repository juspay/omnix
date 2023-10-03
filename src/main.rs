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
            .with_custom_head(r#"<link rel="stylesheet" href="tailwind.css">"#.to_string()),
    )
}

fn App(cx: Scope) -> Element {
    render! {
        body {
        div { class: "flex justify-center w-full min-h-screen bg-center bg-cover bg-base-200",
            div { class: "flex flex-col items-stretch mx-auto sm:container sm:max-w-screen-md",
                main { class: "flex flex-col px-2 mb-8 space-y-3 text-center",
                    Nav {}
                    p { "It is WIP" }
                    ul { li { "Tailwind works" } }
                    img { src: "images/128x128.png" }
                }
            }
        }
    }
    }
}

fn Nav(cx: Scope) -> Element {
    let class = "px-3 py-2";
    render! {
        nav { class: "flex flex-row w-full mb-8 text-white md:rounded-b bg-primary-800",
            a { href: "/", class: class, "Dashboard" }
            a { href: "/flake", class: class, "Flake" }
            a { href: "/health", class: class, "Nix Health" }
            a { href: "/info", class: class, "Nix Info" }
            a { href: "/about", class: class, "About" }
            div { class: "flex-grow font-bold text-end {class}", "🌍 nix-browser" }
        }
    }
}
