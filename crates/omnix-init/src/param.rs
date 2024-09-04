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
        default: Option<String>,
    },
    /// Delete 'files' unless 'default' is true
    Retain {
        /// The file names (suffixes) to retain or delete
        files: Vec<PathBuf>,
        #[serde(default)]
        default: Option<bool>,
    },
}

/// Set 'default' fields of prompts to the user-defined values
///
/// Given a list of prompts, and the user-defined default values for a subset of them (as JSON-parsed `HashMap<String, Value>` where String is the prompt name and serde 'Value' is the 'default' field of action), mutate the prompts to set those 'default' fields
pub fn set_defaults(prompts: &mut [Param], defaults: &HashMap<String, Value>) {
    for prompt in prompts.iter_mut() {
        if let Some(default) = defaults.get(&prompt.name) {
            prompt.set_default(default);
        }
    }
}

impl Param {
    fn set_default(&mut self, val: &Value) {
        match &mut self.action {
            Action::Replace { default, .. } => {
                *default = val.as_str().map(|s| s.to_string());
            }
            Action::Retain { default, .. } => {
                *default = val.as_bool();
            }
        }
    }

    /// Prompt the user for a value for this [Param] using inquire crate.
    pub fn prompt_value(&self) -> anyhow::Result<()> {
        match &self.action {
            Action::Replace {
                placeholder,
                default,
            } => {
                let mut p = inquire::Text::new(&self.name)
                    .with_help_message(&self.description)
                    .with_placeholder(&placeholder);
                if let Some(def) = default.as_ref() {
                    p = p.with_default(def);
                }

                let to = p.prompt()?;
                if !to.is_empty() {
                    println!("Replace '{}' with '{}'", placeholder, to);
                }
            }
            Action::Retain { files, default } => {
                let mut p = inquire::Confirm::new(&self.name).with_help_message(&self.description);
                if let Some(def) = default {
                    p = p.with_default(*def);
                }

                let v = p.prompt()?;
                if v {
                    println!("Retain files: {:?}", files);
                } else {
                    println!("Delete files: {:?}", files);
                }
            }
        }
        Ok(())
    }
}
