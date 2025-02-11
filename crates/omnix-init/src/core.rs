use std::{collections::HashMap, path::Path};

use crate::config::{load_templates, FlakeTemplate};
use anyhow::Context;
use nix_rs::{
    command::NixCmd,
    flake::{system::System, url::FlakeUrl},
};
use omnix_common::markdown::print_markdown;
use serde_json::Value;

pub async fn select_from_registry() -> anyhow::Result<FlakeUrl> {
    let builtin_registry = crate::registry::get().await.as_ref()?;
    // Prompt the user to select a flake from the registry
    let available: Vec<String> = builtin_registry.0.keys().cloned().collect();
    let name = inquire::Select::new("Select a flake", available).prompt()?;
    builtin_registry
        .0
        .get(&name)
        .cloned()
        .ok_or(anyhow::anyhow!("Flake not found in builtin registry"))
}

pub async fn run_tests(
    nixcmd: &NixCmd,
    current_system: &System,
    flake: &FlakeUrl,
) -> anyhow::Result<()> {
    let templates = load_templates(nixcmd, flake).await?;
    for template in templates.iter() {
        tracing::info!("🕍 Testing template: {}#{}", flake, template.template_name);
        for (name, test) in template.template.tests.iter() {
            if test.can_run_on(current_system) {
                tracing::info!("🧪 Running test: {} (on {})", name, current_system);
                test.run_test(
                    &flake.with_attr(&format!("{}.{}", template.template_name, name)),
                    template,
                )
                .await?;
            } else {
                tracing::info!(
                    "⚠️ Skipping test: {} (cannot run on {})",
                    name,
                    current_system
                );
            }
        }
    }
    Ok(())
}

/// Initialize a template at the given path
///
/// # Arguments
/// - `path` - The path to initialize the template at
/// - `name` - The name of the template to initialize
/// - `default_params` - The default parameter values to use
/// - `non_interactive` - Whether to disable user prompts (all params must have values set)
pub async fn run(
    nixcmd: &NixCmd,
    path: &Path,
    flake: &FlakeUrl,
    default_params: &HashMap<String, Value>,
    non_interactive: bool,
) -> anyhow::Result<()> {
    let templates = load_templates(nixcmd, flake).await?;
    // Prompt the user to select a template
    let mut template: FlakeTemplate = if let Some(attr) = flake.get_attr().0 {
        templates
            .iter()
            .find(|t| t.template_name == attr)
            .cloned()
            .with_context(|| "Template not found")?
    } else if templates.len() < 2 {
        if let Some(first) = templates.first() {
            tracing::info!(
                "Automatically choosing the one template available: {}",
                first.template_name
            );
            first.clone()
        } else {
            return Err(anyhow::anyhow!("No templates available"));
        }
    } else if non_interactive {
        return Err(
              anyhow::anyhow!("Non-interactive mode requires exactly one template to be available; but {} are available. Explicit specify it in flake URL.",
              templates.len()));
    } else {
        let select = inquire::Select::new("Select a template", templates);
        select.prompt()?.clone()
    };

    template.template.set_param_values(default_params);

    if non_interactive {
        for param in template.template.params.iter() {
            if !param.action.has_value() {
                return Err(anyhow::anyhow!(
                    "Non-interactive mode requires all parameters to be set; but {} is missing",
                    param.name
                ));
            }
        }
    } else {
        for param in template.template.params.iter_mut() {
            param.set_value_by_prompting()?;
        }
    }

    initialize_template(path, &template).await?;
    Ok(())
}

async fn initialize_template(path: &Path, template: &FlakeTemplate<'_>) -> anyhow::Result<()> {
    tracing::info!("Initializing template at {}", path.display());
    let path = template
        .template
        .scaffold_at(path)
        .await
        .with_context(|| "Unable to scaffold")?;
    eprintln!();
    print_markdown(
        &path,
        &format!("## 🥳 Initialized template at `{}`", path.display()),
    )
    .await?;

    if let Some(welcome_text) = template.template.template.welcome_text.as_ref() {
        eprintln!();
        print_markdown(&path, welcome_text).await?;
    }

    Ok(())
}
