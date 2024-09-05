use std::{collections::HashMap, path::Path};

use crate::config::load_templates;
use anyhow::Context;
use serde_json::Value;

pub async fn initialize_template(
    path: &Path,
    name: Option<String>,
    default_params: &HashMap<String, Value>,
    non_interactive: bool,
) -> anyhow::Result<()> {
    tracing::info!("Loading registry");
    let templates = load_templates().await?;

    let mut template = match name {
        Some(name) => templates
            .get(&name)
            .cloned()
            .with_context(|| "Template not found")?,
        None => {
            // select a template by prompting the user using inquire crate
            let p = inquire::Select::new("Select a template", templates.keys().cloned().collect());
            let name = p.prompt()?;
            templates.get(&name).cloned().unwrap()
        }
    };

    template.set_param_values(default_params);

    if non_interactive {
        for param in template.params.iter() {
            if !param.action.has_value() {
                return Err(anyhow::anyhow!(
                    "Non-interactive mode requires all parameters to be set; but {} is missing",
                    param.name
                ));
            }
        }
    } else {
        for param in template.params.iter_mut() {
            param.set_value_by_prompting()?;
        }
    }

    template
        .scaffold_at(path)
        .await
        .with_context(|| "Unable to scaffold")?;

    // print welcomeText
    if let Some(welcome_text) = template.template.welcome_text {
        tracing::info!("\n{}", welcome_text);
    }

    Ok(())
}
