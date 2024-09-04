use std::{collections::HashMap, path::Path};

use anyhow::Context;
use omnix_init::config::load_templates;
use serde_json::Value;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("Loading registry");
    let templates = load_templates().await?;

    // select a template by prompting the user using inquire crate
    let p = inquire::Select::new("Select a template", templates.keys().cloned().collect());
    let template = p.prompt()?;
    let mut template = templates.get(template.as_str()).cloned().unwrap();
    println!("Selected template: {:?}", template);

    // TODO: dummy
    let defaults: HashMap<String, Value> =
        serde_json::from_str(r#"{"git-email": "srid@srid.ca", "param2": true}"#)?;

    template.set_param_values(&defaults);
    for param in template.params.iter_mut() {
        param.prompt_and_set_value()?;
    }

    template
        .scaffold_at(Path::new("/tmp/init"))
        .await
        .with_context(|| "Unable to scaffold")?;

    // print welcomeText
    if let Some(welcome_text) = template.template.welcome_text {
        println!("\n---\n{}\n---", welcome_text);
    }

    Ok(())
}
