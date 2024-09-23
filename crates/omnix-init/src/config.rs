use std::collections::HashMap;

use nix_rs::{
    command::NixCmd,
    flake::{command::FlakeOptions, eval::nix_eval, url::FlakeUrl},
};

use crate::template::Template;

/// The `om.templates` config in flake
pub type TemplatesConfig = HashMap<String, Template>;

/// Load templates from the given flake
pub async fn load_templates(url: &FlakeUrl) -> anyhow::Result<TemplatesConfig> {
    let v = nix_eval::<TemplatesConfig>(
        &NixCmd::default(),
        &FlakeOptions::default(),
        &url.with_attr("om.templates"),
    )
    .await?;
    Ok(v)
}
