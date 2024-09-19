use std::{collections::HashMap, path::Path};

use crate::config::{load_templates, BUILTIN_REGISTRY};
use anyhow::Context;
use nix_rs::flake::url::FlakeUrl;
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
    flake: Option<FlakeUrl>,
    default_params: &HashMap<String, Value>,
    non_interactive: bool,
) -> anyhow::Result<()> {
    tracing::info!("Loading registry ...");
    let registry = flake.unwrap_or(BUILTIN_REGISTRY.clone());
    let templates = load_templates(&registry).await?;

    // Prompt the user to select a template
    let available: Vec<String> = templates.keys().cloned().collect();
    let name: &String = match registry.get_attr().0 {
        Some(attr) => &attr.clone(),
        None => {
            if available.len() < 2 {
                if let Some(name) = available.first() {
                    tracing::info!(
                        "Automatically choosing the one template available: {}",
                        name
                    );
                    name
                } else {
                    return Err(anyhow::anyhow!("No templates available"));
                }
            } else {
                if non_interactive {
                    return Err(anyhow::anyhow!("Non-interactive mode requires exactly one template to be available; but {} are available. Explicit specify it in flake URL.", available.len()));
                } else {
                    &inquire::Select::new("Select a template", available).prompt()?
                }
            }
        }
    };

    let mut template = templates
        .get(name)
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
