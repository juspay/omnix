use std::{
    collections::HashMap,
    fmt::{self, Display, Formatter},
    path::Path,
};

use crate::config::load_templates;
use anyhow::Context;
use colored::Colorize;
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
    m_flake: Option<FlakeUrl>,
    default_params: &HashMap<String, Value>,
    non_interactive: bool,
) -> anyhow::Result<()> {
    let flake: FlakeUrl = match m_flake {
        Some(flake) => Ok(flake),
        None => {
            let builtin_registry = crate::registry::BUILTIN_REGISTRY.clone();
            // Prompt the user to select a flake from the registry
            let available: Vec<String> = builtin_registry.0.keys().cloned().collect();
            let name = inquire::Select::new("Select a flake", available).prompt()?;
            builtin_registry
                .0
                .get(&name)
                .cloned()
                .ok_or(anyhow::anyhow!("Flake not found in builtin registry"))
        }
    }?;
    let templates = load_templates(&flake).await?;

    // Prompt the user to select a template
    // let available: Vec<String> = templates.keys().cloned().collect();
    let available: Vec<AssocTemplate> = templates
        .keys()
        .into_iter()
        .map(|k| AssocTemplate {
            flake: &flake,
            template_name: k.as_str(),
        })
        .collect();
    let name: &str = if let Some(attr) = flake.get_attr().0 {
        &attr.clone()
    } else if available.len() < 2 {
        if let Some(first) = available.first() {
            tracing::info!(
                "Automatically choosing the one template available: {}",
                first.template_name
            );
            first.template_name
        } else {
            return Err(anyhow::anyhow!("No templates available"));
        }
    } else if non_interactive {
        return Err(anyhow::anyhow!("Non-interactive mode requires exactly one template to be available; but {} are available. Explicit specify it in flake URL.", available.len()));
    } else {
        let select = inquire::Select::new("Select a template", available);
        select.prompt()?.template_name
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
        let skin = termimad::MadSkin::default();
        eprint!(
            "\n{}",
            skin.term_text(&format!("---\n{}---", &welcome_text))
        );
    }

    Ok(())
}

/// A template associated with a flake
#[derive(Debug, Clone, PartialEq, Eq)]
struct AssocTemplate<'a> {
    flake: &'a FlakeUrl,
    template_name: &'a str,
}

impl<'a> Display for AssocTemplate<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:<15} {}",
            self.template_name,
            format!("[{}]", self.flake).dimmed()
        )
    }
}
