#![feature(lazy_cell)]
#![feature(let_chains)]
pub mod flake_template;
pub mod registry;

use std::path::PathBuf;

use nix_rs::command::NixCmd;
use nix_rs::flake::url::FlakeUrl;

use crate::{flake_template::fileop::FileOp, registry::TemplateRegistryRef};

pub async fn flakreate(registry: FlakeUrl, path: PathBuf) -> anyhow::Result<()> {
    tracing::info!("Let's create your flake template at {:?}:", path);
    let template = TemplateRegistryRef::from_url(registry.clone())?
        .load_and_select_template()
        .await?;

    // Prompt for template parameters
    let param_values = template.prompt_replacements()?;

    let path = path.to_string_lossy();

    // Create the flake template
    let template_url = registry.with_attr(&format!("om.templates.{}", template.name));
    NixCmd::get()
        .await
        .run_with(|cmd| {
            cmd.args(["flake", "new", &path, "-t", &template_url.0]);
        })
        .await?;

    // Do the actual replacement
    std::env::set_current_dir(&*path)?;
    for replace in param_values {
        FileOp::apply(&replace).await?;
    }
    Ok(())
}
