use std::{collections::BTreeMap, path::PathBuf};

use glob::{Pattern, PatternError};
use inquire::Select;
use nix_rs::{
    command::{NixCmd, NixCmdError},
    flake::{eval::nix_eval_attr, url::FlakeUrl},
};
use thiserror::Error;

use crate::flake_template::template::FlakeTemplate;

/// A [FlakeUrl] reference to a [TemplateRegistry]
pub struct TemplateRegistryRef {
    pub flake_url: FlakeUrl,
    /// A filter to apply to the templates. Unmatched templates will be ignored.
    pub filter: Option<Pattern>,
}

impl TemplateRegistryRef {
    pub fn from_url(url: FlakeUrl) -> Result<Self, PatternError> {
        let (base, attr) = url.split_attr();
        Ok(TemplateRegistryRef {
            flake_url: base,
            filter: if attr.is_none() {
                None
            } else {
                Some(Pattern::new(&attr.get_name())?)
            },
        })
    }

    pub async fn load_and_select_template(&self) -> anyhow::Result<FlakeTemplate> {
        tracing::info!("Loading registry {}...", self.flake_url);
        let templates = self.load_registry().await?;
        // TODO: avoid duplicates (aliases)
        let filtered_templates = templates
            .0
            .iter()
            .filter(|template| {
                self.filter
                    .as_ref()
                    .map_or(true, |filter| filter.matches(&template.name))
            })
            .collect::<Vec<_>>();
        let template = if filtered_templates.len() == 1 {
            filtered_templates[0]
        } else {
            Select::new("Select a template", filtered_templates)
                .with_help_message("Choose a flake template to use")
                .prompt()?
        };
        tracing::info!("Selected template: {}", template);
        Ok(template.clone())
    }

    async fn load_registry(&self) -> anyhow::Result<TemplateRegistry> {
        let res = TemplateRegistry::from(&self.flake_url).await?;
        Ok(res)
    }
}

/// A registry of [FlakeTemplate]s
pub struct TemplateRegistry(pub Vec<FlakeTemplate>);

impl TemplateRegistry {
    /// Fetch the templates defined in a flake
    pub async fn from(url: &FlakeUrl) -> Result<Self, TemplateError> {
        let v = if let Some(path) = url.as_local_path()
            && let cache_file = path.join("flake.nix.json")
            && cache_file.exists()
        {
            tracing::debug!("Fetching templates from cache: {}", cache_file.display());
            Self::fetch_via_cache(cache_file).await?
        } else {
            tracing::debug!("Fetching templates from flake: {}", url);
            Self::fetch_via_flake(url).await?
        };
        Ok(v)
    }

    async fn fetch_via_flake(url: &FlakeUrl) -> Result<Self, NixCmdError> {
        let nixcmd = NixCmd::get().await;
        let mut templates = nix_eval_attr::<BTreeMap<String, FlakeTemplate>>(
            nixcmd,
            &url.with_attr("om.templates"),
        )
        .await?
        .unwrap_or_default();
        Self::set_template_deserialized_fields(&mut templates);
        Ok(TemplateRegistry(templates.values().cloned().collect()))
    }

    /// Load from `flake.nix.json`
    async fn fetch_via_cache(cache_file: PathBuf) -> Result<Self, CacheError> {
        let flake_nix_json: serde_json::Value =
            serde_json::from_reader(std::fs::File::open(cache_file)?)?;
        let mut templates = serde_json::from_value(
            flake_nix_json
                .pointer("/om/templates")
                .cloned()
                .unwrap_or_default(),
        )?;
        Self::set_template_deserialized_fields(&mut templates);
        Ok(TemplateRegistry(templates.values().cloned().collect()))
    }

    fn set_template_deserialized_fields(templates: &mut BTreeMap<String, FlakeTemplate>) {
        for (name, template) in templates.iter_mut() {
            template.polyfill(name.clone());
        }
    }
}

#[derive(Error, Debug)]
pub enum TemplateError {
    #[error("Failed to fetch templates: {0}")]
    FetchError(#[from] NixCmdError),

    #[error("Failed to fetch templates from cache: {0}")]
    CacheError(#[from] CacheError),
}

#[derive(Error, Debug)]
pub enum CacheError {
    #[error("Failed to read flake.nix.json: {0}")]
    ReadError(#[from] std::io::Error),

    #[error("Failed to parse flake.nix.json: {0}")]
    ParseError(#[from] serde_json::Error),
}
