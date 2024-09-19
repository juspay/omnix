use std::{collections::HashMap, path::Path};

use crate::config::load_templates;
use anyhow::Context;
use serde_json::Value;

/// Initialize a template at the given path
///
/// # Arguments
/// - `path` - The path to initialize the template at
/// - `name` - The name of the template to initialize
/// - `default_params` - The default parameter values to use
/// - `non_interactive` - Whether to disable user prompts (all params must have values set)
pub async fn initialize_template(
    path: &Path,
    name: Option<String>,
    default_params: &HashMap<String, Value>,
    non_interactive: bool,
) -> anyhow::Result<()> {
    tracing::info!("Loading registry ...");
    let templates = load_templates().await?;

    // If the name is not provided, prompt the user to select a template
    let name = match name {
        Some(name) => name,
        None => {
            let p = inquire::Select::new("Select a template", templates.keys().cloned().collect());
            p.prompt()?
        }
    };

    let mut template = templates
        .get(&name)
        .cloned()
        .with_context(|| "Template not found")?;

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

    tracing::info!("Initializing '{}' template at {}", name, path.display());
    template
        .scaffold_at(path)
        .await
        .with_context(|| "Unable to scaffold")?;
    tracing::info!("🥳 Initialized a {} project at {}", name, path.display());

    if let Some(welcome_text) = template.template.welcome_text {
        tracing::info!("\n{}", welcome_text);
    }

    Ok(())
}
