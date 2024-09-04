use std::{collections::HashMap, path::PathBuf, sync::LazyLock};

use nix_rs::{
    command::NixCmd,
    flake::{eval::nix_eval_attr, url::FlakeUrl},
};

use crate::template::Template;

/// Our builtin registry of templates
static REGISTRY: LazyLock<FlakeUrl> =
    LazyLock::new(|| PathBuf::from(env!("OM_INIT_REGISTRY")).into());

/// The `om.templates` config in flake
pub type TemplatesConfig = HashMap<String, Template>;

/// Load templates from our builtin registry [REGISTRY]
pub async fn load_templates() -> anyhow::Result<TemplatesConfig> {
    let registry = REGISTRY.clone();
    match nix_eval_attr::<TemplatesConfig>(&NixCmd::default(), &registry.with_attr("om.templates"))
        .await?
    {
        Some(v) => Ok(v),
        None => Ok(HashMap::new()),
    }
}
