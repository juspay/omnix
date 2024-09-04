use std::collections::HashMap;

use omnix_init::param;
use serde_json::Value;

fn main() -> anyhow::Result<()> {
    let mut params: Vec<param::Param> = serde_json::from_str(include_str!("demo.json"))?;
    let defaults: HashMap<String, Value> =
        serde_json::from_str(r#"{"git-email": "srid@srid.ca", "param2": true}"#)?;

    param::set_defaults(&mut params, &defaults);
    for param in params.iter() {
        param.prompt_value()?;
    }

    Ok(())
}
