use glob::{Pattern, PatternError};
use inquire::Select;
use nix_rs::flake::url::FlakeUrl;

use crate::flake_template::{self, template::FlakeTemplate};
use clap::Parser;

#[derive(Parser)]
pub struct FlakeTemplateRegistry {
    #[arg(value_hint = clap::ValueHint::AnyPath)]
    pub flake_url: FlakeUrl,
    pub filter: Option<Pattern>,
}

impl FlakeTemplateRegistry {
    pub fn from_url(url: FlakeUrl) -> Result<Self, PatternError> {
        let (base, attr) = url.split_attr();
        Ok(FlakeTemplateRegistry {
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

    async fn load_registry(&self) -> anyhow::Result<Vec<FlakeTemplate>> {
        let res = flake_template::template::fetch(&self.flake_url).await?;
        Ok(res)
    }
}
