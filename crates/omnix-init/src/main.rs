use std::collections::HashMap;

use omnix_init::config::*;
use serde_json::Value;

fn main() -> anyhow::Result<()> {
    let mut prompts: Vec<Param> = serde_json::from_str(include_str!("demo.json"))?;
    let defaults: HashMap<String, Value> =
        serde_json::from_str(r#"{"git-email": "srid@srid.ca", "param2": true}"#)?;

    set_defaults(&mut prompts, &defaults);
    for prompt in prompts.iter() {
        prompt_value(prompt)?;
    }

    Ok(())
}
