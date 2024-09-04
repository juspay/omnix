use anyhow::Context;
use itertools::Itertools;
use serde::Deserialize;
use serde_json::Value;
use std::collections::HashMap;
use std::path::Path;

use crate::action::Action;

#[derive(Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Param {
    pub name: String,
    pub description: String,
    #[serde(flatten)]
    pub action: Action,
}

/// Set 'default' fields of prompts to the user-defined values
///
/// Given a list of prompts, and the user-defined default values for a subset of them (as JSON-parsed `HashMap<String, Value>` where String is the prompt name and serde 'Value' is the 'default' field of action), mutate the prompts to set those 'default' fields
pub fn set_values(prompts: &mut [Param], values: &HashMap<String, Value>) {
    for prompt in prompts.iter_mut() {
        if let Some(v) = values.get(&prompt.name) {
            prompt.set_value(v);
        }
    }
}

pub async fn apply_actions(params: &[Param], out_dir: &Path) -> anyhow::Result<()> {
    for param in params.iter().sorted_by(|a, b| a.action.cmp(&b.action)) {
        println!("Applying param: {:?}", param);
        param
            .action
            .apply(out_dir.as_ref())
            .await
            .with_context(|| format!("Unable to apply param {}", param.name))?;
    }
    Ok(())
}

impl Param {
    fn set_value(&mut self, val: &Value) {
        match &mut self.action {
            Action::Replace { value, .. } => {
                *value = val.as_str().map(|s| s.to_string());
            }
            Action::Retain { value, .. } => {
                *value = val.as_bool();
            }
        }
    }

    /// Prompt the user for a value for this [Param] using inquire crate.
    pub fn prompt_and_set_value(&mut self) -> anyhow::Result<()> {
        match &mut self.action {
            Action::Replace { placeholder, value } => {
                let mut p = inquire::Text::new(&self.description).with_placeholder(placeholder);
                if let Some(def) = value.as_ref() {
                    p = p.with_default(def);
                }

                let to = p.prompt()?;
                if !to.is_empty() {
                    println!("Replace '{}' with '{}'", placeholder, to);
                    *value = Some(to);
                }
            }
            Action::Retain { paths, value } => {
                let mut p = inquire::Confirm::new(&self.description);
                if let Some(def) = value {
                    p = p.with_default(*def);
                }

                let v = p.prompt()?;
                if v {
                    println!("Retain paths: {:?}", paths);
                } else {
                    println!("Delete paths: {:?}", paths);
                }
                *value = Some(v)
            }
        }
        Ok(())
    }
}
