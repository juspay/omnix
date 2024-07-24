#![feature(lazy_cell)]
pub mod flake_template;
pub mod registry;

use std::path::PathBuf;

use nix_rs::command::NixCmd;
use nix_rs::flake::url::FlakeUrl;
use tokio::sync::OnceCell;

use crate::{flake_template::fileop::FileOp, registry::FlakeTemplateRegistry};

static NIXCMD: OnceCell<NixCmd> = OnceCell::const_new();

/// TODO: Can we normalize this across omnix-cli?
async fn nixcmd() -> &'static NixCmd {
    NIXCMD
        .get_or_init(|| async { NixCmd::default().with_flakes().await.unwrap() })
        .await
}

pub async fn flakreate(registry: FlakeUrl, path: PathBuf) -> anyhow::Result<()> {
    println!(
        "Welcome to flakreate! Let's create your flake template at {:?}:",
        path
    );
    let template = FlakeTemplateRegistry::from_url(registry.clone())?
        .load_and_select_template()
        .await?;

    // Prompt for template parameters
    let param_values = template.prompt_replacements()?;

    let path = path.to_string_lossy();

    // Create the flake templatge
    let template_url = registry.with_attr(&template.name);
    println!("$ nix flake new {} -t {}", path, template_url);
    nixcmd()
        .await
        .run_with_args(&["flake", "new", &path, "-t", &template_url.0])
        .await?;

    // Do the actual replacement
    std::env::set_current_dir(&*path)?;
    for replace in param_values {
        FileOp::apply(&replace).await?;
    }
    Ok(())
}
