use serde::Deserialize;
use std::cmp::Ordering;
use std::path::{Path, PathBuf};
use tokio::fs;

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