use std::path::PathBuf;

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
