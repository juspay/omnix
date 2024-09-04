use glob::Pattern;
use itertools::Itertools;
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
                    let files = omnix_common::fs::find_files(out_dir).await?;
                    // Build glob patterns from paths
                    let mut pats = vec![];
                    for path in paths.iter() {
                        let path = path.to_string_lossy();
                        pats.push(Pattern::new(path.as_ref())?);
                    }
                    let files_to_delete = files
                        .iter()
                        .filter(|file| pats.iter().any(|pat| pat.matches_path(file)))
                        .collect::<Vec<_>>();
                    if files_to_delete.is_empty() {
                        anyhow::bail!("No paths matched in {:?}", files);
                    };
                    // Iterating in reverse-sorted order ensures that children gets deleted before their parent folders.
                    for file in files_to_delete.iter().sorted().rev() {
                        let path = out_dir.join(file);
                        println!("Deleting path {:?}", path);
                        if path.is_dir() {
                            fs::remove_dir(path).await?;
                        } else {
                            fs::remove_file(path).await?;
                        }
                    }
                }
            }
        }
        Ok(())
    }
}
