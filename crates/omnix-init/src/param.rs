use serde::Deserialize;
use serde_json::Value;
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Deserialize, Debug, Clone)]
pub struct Param {
    name: String,
    description: String,
    #[serde(flatten)]
    action: Action,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum Action {
    /// Replace 'placeholder' with 'default'
    Replace {
        /// The text to replace.
        placeholder: String,
        /// The text to replace it with.
        #[serde(default)]
        value: Option<String>,
    },
    /// Delete 'files' unless 'default' is true
    Retain {
        /// The file names (suffixes) to retain or delete
        paths: Vec<PathBuf>,
        #[serde(default)]
        value: Option<bool>,
    },
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
    pub fn prompt_value(&self) -> anyhow::Result<Option<Action>> {
        match &self.action {
            Action::Replace { placeholder, value } => {
                let mut p = inquire::Text::new(&self.description).with_placeholder(placeholder);
                if let Some(def) = value.as_ref() {
                    p = p.with_default(def);
                }

                let to = p.prompt()?;
                if !to.is_empty() {
                    println!("Replace '{}' with '{}'", placeholder, to);
                    Ok(Some(Action::Replace {
                        placeholder: placeholder.clone(),
                        value: Some(to),
                    }))
                } else {
                    Ok(None)
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
                Ok(Some(Action::Retain {
                    paths: paths.clone(),
                    value: Some(v),
                }))
            }
        }
    }
}
