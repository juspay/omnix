use dioxus_desktop::WindowBuilder;

mod app;
mod cli;
mod logging;
mod widget;

#[tokio::main]
async fn main() {
    use clap::Parser;
    let args = crate::cli::Args::parse();
    crate::logging::setup_logging(&args.verbosity);

    dioxus_desktop::launch_cfg(
        app::App,
        dioxus_desktop::Config::new()
            .with_custom_head(r#" <link rel="stylesheet" href="tailwind.css"> "#.to_string())
            .with_window(WindowBuilder::new().with_title("nix-browser")),
    )
}
