use std::{collections::HashMap, path::Path};

use crate::config::load_templates;
use anyhow::Context;
use serde_json::Value;

pub async fn initialize_template(
    path: &Path,
    name: Option<String>,
    default_params: &HashMap<String, Value>,
) -> anyhow::Result<()> {
    println!("Loading registry");
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
    println!("Selected template: {:?}", template);

    template.set_param_values(default_params);
    for param in template.params.iter_mut() {
        param.set_value_by_prompting()?;
    }

    template
        .scaffold_at(path)
        .await
        .with_context(|| "Unable to scaffold")?;

    // print welcomeText
    if let Some(welcome_text) = template.template.welcome_text {
        println!("\n---\n{}\n---", welcome_text);
    }

    Ok(())
}
