#![feature(let_chains)]
use dioxus_desktop::{LogicalSize, WindowBuilder};
use directories::ProjectDirs;

mod app;
mod cli;
mod logging;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    use clap::Parser;
    let args = crate::cli::Args::parse();
    crate::logging::setup_logging(&args.verbosity);

    let data_dir = ProjectDirs::from("in", "juspay", "nix-browser")
        .ok_or(anyhow::anyhow!("Unable to deduce ProjectDirs"))?
        .data_local_dir()
        .to_path_buf();

    tracing::info!("Data dir: {:?}", data_dir);

    dioxus_desktop::launch_cfg(
        app::App,
        dioxus_desktop::Config::new()
            .with_custom_head(r#" <link rel="stylesheet" href="tailwind.css"> "#.to_string())
            .with_window(
                WindowBuilder::new()
                    .with_title("Nix Browser")
                    .with_inner_size(LogicalSize::new(800, 700)),
            ),
    );

    Ok(())
}
