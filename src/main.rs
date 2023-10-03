mod app;
mod cli;
mod logging;
mod widget;

#[tokio::main]
async fn main() {
    // use clap::Parser;
    // let args = crate::cli::Args::parse();

    // FIXME: remove dioxus:/ for release
    dioxus_desktop::launch_cfg(
        app::App,
        dioxus_desktop::Config::new()
            .with_custom_head(r#"<link rel="stylesheet" href="tailwind.css">"#.to_string()),
    )
}
