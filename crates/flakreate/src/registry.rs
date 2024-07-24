use glob::{Pattern, PatternError};
use inquire::Select;
use nix_rs::flake::url::FlakeUrl;

use crate::flake_template::{self, template::FlakeTemplate};

pub struct FlakeTemplateRegistry {
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
        let term = console::Term::stdout();
        term.write_line(format!("Loading registry {}...", self.flake_url).as_str())?;
        let templates = flake_template::template::fetch(&self.flake_url).await?;
        term.clear_last_lines(1)?;
        println!("Loaded registry: {}", self.flake_url);
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
        println!("Selected template: {}", template);
        Ok(template.clone())
    }
}
