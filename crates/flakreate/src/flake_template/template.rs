use colored::Colorize;
use core::fmt;
use std::{
    collections::BTreeMap,
    fmt::{Display, Formatter},
    path::PathBuf,
};
use thiserror::Error;

use nix_rs::{
    command::{NixCmd, NixCmdError},
    flake::url::FlakeUrl,
};
use serde::{Deserialize, Serialize};

use super::{config::FlakeTemplateConfig, fileop::FileOp};

/// A Nix flake template
///
/// Defined per [this definition](https://nix.dev/manual/nix/2.22/command-ref/new-cli/nix3-flake-init#template-definitions) in the flake.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FlakeTemplate {
    #[serde(skip_deserializing)]
    pub name: String,

    pub description: String,

    pub path: String,

    #[serde(rename = "welcomeText")]
    pub welcome_text: Option<String>,

    #[serde(skip_deserializing)]
    pub config: FlakeTemplateConfig,
}

impl Display for FlakeTemplate {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if self.config.tags.is_empty() {
            write!(f, "{}", self.name)
        } else {
            write!(
                f,
                "{:<20} {}",
                self.name,
                self.config.tags.join(", ").dimmed()
            )
        }
    }
}

impl FlakeTemplate {
    pub fn prompt_replacements(&self) -> anyhow::Result<Vec<Vec<FileOp>>> {
        self.config
            .params
            .iter()
            .map(|param| param.prompt_value())
            .collect()
    }
}

/// Fetch the templates defined in a flake
pub async fn fetch(url: &FlakeUrl) -> Result<Vec<FlakeTemplate>, TemplateError> {
    let v = if let Some(path) = url.as_local_path()
        && let cache_file = path.join("flake.nix.json")
        && cache_file.exists()
    {
        tracing::debug!("Fetching templates from cache: {}", cache_file.display());
        fetch_via_cache(cache_file).await?
    } else {
        tracing::debug!("Fetching templates from flake: {}", url);
        fetch_via_flake(url).await?
    };
    Ok(v)
}

async fn fetch_via_flake(url: &FlakeUrl) -> Result<Vec<FlakeTemplate>, NixCmdError> {
    let nixcmd = NixCmd::get().await;
    let mut templates = nix_rs::flake::eval::nix_eval_attr_json::<BTreeMap<String, FlakeTemplate>>(
        nixcmd,
        &url.with_attr("templates"),
    )
    .await?
    .unwrap_or_default();
    let templates_config = nix_rs::flake::eval::nix_eval_attr_json::<
        BTreeMap<String, FlakeTemplateConfig>,
    >(nixcmd, &url.with_attr("om.templates"))
    .await?
    .unwrap_or_default();
    set_template_deserialized_fields(&mut templates, &templates_config);
    Ok(templates.values().cloned().collect())
}

/// Load from `flake.nix.json`
async fn fetch_via_cache(cache_file: PathBuf) -> Result<Vec<FlakeTemplate>, CacheError> {
    let flake_nix_json: serde_json::Value =
        serde_json::from_reader(std::fs::File::open(cache_file)?)?;
    let mut templates = serde_json::from_value(
        flake_nix_json
            .pointer("/templates")
            .cloned()
            .unwrap_or_default(),
    )?;
    let templates_config = serde_json::from_value(
        flake_nix_json
            .pointer("/om/templates")
            .cloned()
            .unwrap_or_default(),
    )?;
    set_template_deserialized_fields(&mut templates, &templates_config);
    Ok(templates.values().cloned().collect())
}

fn set_template_deserialized_fields(
    templates: &mut BTreeMap<String, FlakeTemplate>,
    templates_config: &BTreeMap<String, FlakeTemplateConfig>,
) {
    for (name, template) in templates.iter_mut() {
        // Set 'name' field in each template
        template.name.clone_from(name);
        // Pull in `om.templates` configuration
        template.config = templates_config.get(name).cloned().unwrap_or_default();
    }
}

#[derive(Error, Debug)]
pub enum CacheError {
    #[error("Failed to read flake.nix.json: {0}")]
    ReadError(#[from] std::io::Error),

    #[error("Failed to parse flake.nix.json: {0}")]
    ParseError(#[from] serde_json::Error),
}

#[derive(Error, Debug)]
pub enum TemplateError {
    #[error("Failed to fetch templates: {0}")]
    FetchError(#[from] NixCmdError),

    #[error("Failed to fetch templates from cache: {0}")]
    CacheError(#[from] CacheError),
}
