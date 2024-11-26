use std::fmt::{self, Display, Formatter};

use colored::Colorize;
use nix_rs::{command::NixCmd, flake::url::FlakeUrl};
use omnix_common::config::OmConfig;

use crate::template::Template;

/// A named [Template] associated with a [FlakeUrl]
#[derive(Debug, Clone)]
pub struct FlakeTemplate<'a> {
    pub flake: &'a FlakeUrl,
    pub template_name: String,
    pub template: Template,
}

// This instance is used during user prompting.
impl<'a> Display for FlakeTemplate<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:<15} {} {}",
            self.template_name,
            format!("[{}]", self.flake).dimmed(),
            self.template
                .template
                .description
                .as_ref()
                .unwrap_or(&"".to_string())
        )
    }
}

/// Load templates from the given flake
pub async fn load_templates<'a>(url: &FlakeUrl) -> anyhow::Result<Vec<FlakeTemplate>> {
    let om_config = OmConfig::get(NixCmd::get().await, url).await?;

    let templates = om_config
        .config
        .get::<Template>("templates")?
        .ok_or_else(|| anyhow::anyhow!("No templates found"))?;

    Ok(templates
        .into_iter()
        .map(|(k, v)| FlakeTemplate {
            flake: url,
            template_name: k,
            template: v,
        })
        .collect())
}
