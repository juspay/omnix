use anyhow::Context;
use itertools::Itertools;
use serde::Deserialize;
use serde_json::Value;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tokio::fs;

#[derive(Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Param {
    pub name: String,
    pub description: String,
    #[serde(flatten)]
    pub action: Action,
}

#[derive(Deserialize, Debug, Clone, PartialEq, Eq)]
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

// Implement Ord such that 'Retain' appears before 'Replace' in the list
// Because, Retain will delete files, which affect the Replace actions.
impl Ord for Action {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (Action::Retain { .. }, Action::Replace { .. }) => Ordering::Less,
            (Action::Replace { .. }, Action::Retain { .. }) => Ordering::Greater,
            _ => Ordering::Equal,
        }
    }
}

impl PartialOrd for Action {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
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

impl Action {
    /// Apply the [Action] to the given directory
    pub async fn apply(&self, out_dir: &Path) -> anyhow::Result<()> {
        match &self {
            Action::Replace { placeholder, value } => {
                if let Some(value) = value.as_ref() {
                    let files = omnix_common::fs::find_files(out_dir).await?;

                    // Replace in content of files
                    for file in files.iter() {
                        if file.is_file() {
                            let content = fs::read_to_string(file).await?;
                            if content.contains(placeholder) {
                                println!(
                                    "Replacing '{}' with '{}' in {:?}",
                                    placeholder, value, file
                                );
                                let content = content.replace(placeholder, value);
                                fs::write(file, content).await?;
                            }
                        }
                    }
                    // Replace in filename of files
                    for file in files.iter() {
                        if let Some(file_name) = file.file_name().map(|f| f.to_string_lossy()) {
                            if file_name.contains(placeholder) {
                                let new_name = file_name.replace(placeholder, value);
                                let new_path = file.with_file_name(new_name);
                                if file != &new_path {
                                    println!(
                                        "Renaming '{}' to '{}'",
                                        file.display(),
                                        new_path.display()
                                    );
                                    fs::rename(file, new_path).await?;
                                }
                            }
                        }
                    }
                }
            }
            Action::Retain { paths, value } => {
                if *value == Some(false) {
                    // TODO: `paths` are just suffixes; this should match that.
                    for path in paths.iter() {
                        let path = out_dir.join(path);
                        if path.exists() {
                            fs::remove_file(path).await?;
                        }
                    }
                }
            }
        }
        Ok(())
    }
}
