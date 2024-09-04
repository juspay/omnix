use std::path::{Path, PathBuf};

use serde::Deserialize;

use crate::param;

/// A template in the `om.templates` config
#[derive(Debug, Deserialize, Clone)]
pub struct Template {
    pub template: NixTemplate,
    pub params: Vec<param::Param>,
}

/// The official Nix template (`flake.templates.<name>`)
#[derive(Debug, Deserialize, Clone)]
pub struct NixTemplate {
    pub path: PathBuf,
    pub description: Option<String>,
    #[serde(rename = "welcomeText")]
    pub welcome_text: Option<String>,
}

impl Template {
    // Scaffold the [Template] at the given path.
    pub async fn scaffold_at(&self, out_dir: impl AsRef<Path>) -> anyhow::Result<()> {
        // Recursively copy the self.template.path to the output directory
        omnix_common::fs::copy_dir_all(&self.template.path, out_dir).await?;
        Ok(())
    }
}
