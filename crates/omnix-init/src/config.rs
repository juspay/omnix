use std::{collections::HashMap, path::PathBuf, sync::LazyLock};

use nix_rs::{
    command::NixCmd,
    flake::{command::FlakeOptions, eval::nix_eval, url::FlakeUrl},
};

use crate::template::Template;

/// Our builtin registry of templates
static REGISTRY: LazyLock<FlakeUrl> =
    LazyLock::new(|| PathBuf::from(env!("OM_INIT_REGISTRY")).into());

/// The `om.templates` config in flake
pub type TemplatesConfig = HashMap<String, Template>;

/// Load templates from our builtin registry `REGISTRY`
pub async fn load_templates() -> anyhow::Result<TemplatesConfig> {
    let v = nix_eval::<TemplatesConfig>(
        &NixCmd::default(),
        &FlakeOptions::default(),
        &REGISTRY.with_attr("om.templates"),
    )
    .await?;
    Ok(v)
}
