use std::collections::HashMap;

use omnix_init::{config::load_templates, param};
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

    let defaults: HashMap<String, Value> =
        serde_json::from_str(r#"{"git-email": "srid@srid.ca", "param2": true}"#)?;

    param::set_values(&mut template.params, &defaults);
    for param in template.params.iter() {
        let _action = param.prompt_value()?;
    }

    Ok(())
}
