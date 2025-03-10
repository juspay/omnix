use anyhow::Context;
use globset::{Glob, GlobSetBuilder};
use itertools::Itertools;
use serde::Deserialize;
use std::cmp::Ordering;
use std::fmt::{self, Display, Formatter};
use std::path::Path;
use tokio::fs;

/// The action to perform on a template
#[derive(Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(untagged)]
pub enum Action {
    /// Replace 'placeholder' with 'value' if it exists
    Replace {
        /// The text to replace.
        placeholder: String,
        /// The text to replace it with.
        #[serde(default)]
        value: Option<String>,
    },
    /// Delete given paths if 'value' is false
    Retain {
        /// The glob patterns to retain or delete
        paths: Vec<Glob>,
        /// Whether to retain or delete
        #[serde(default)]
        value: Option<bool>,
    },
}

impl Display for Action {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Action::Replace { placeholder, value } => match value {
                Some(value) => write!(f, "replace [{} => {}]", placeholder, value),
                None => write!(f, "replace [disabled]"),
            },
            Action::Retain { paths, value } => match value {
                Some(false) => {
                    let paths = paths.iter().map(|p| p.to_string()).join(", ");
                    write!(f, "prune [{}]", paths)
                }
                _ => write!(f, "prune [disabled]"),
            },
        }
    }
}

impl Action {
    /// Whether there is a current value in this action
    pub fn has_value(&self) -> bool {
        match self {
            Action::Replace { value, .. } => value.is_some(),
            Action::Retain { value, .. } => value.is_some(),
        }
    }

    /// Apply the [Action] to the given directory
    pub async fn apply(&self, out_dir: &Path) -> anyhow::Result<()> {
        match &self {
            Action::Replace { placeholder, value } => {
                if let Some(value) = value.as_ref() {
                    let files = omnix_common::fs::find_paths(out_dir).await?;

                    // Process files in reverse order, such that we replace in
                    // files *before* their ancestor directories get renamed.
                    for file in files.iter().sorted().rev() {
                        let file_path = &out_dir.join(file);

                        // Replace in content of files
                        if file_path.is_file() {
                            let content =
                                fs::read_to_string(&file_path).await.with_context(|| {
                                    format!("Unable to read file: {:?}", &file_path)
                                })?;
                            if content.contains(placeholder) {
                                tracing::info!("   ✍️ {}", file.to_string_lossy());
                                let content = content.replace(placeholder, value);
                                fs::write(file_path, content).await?;
                            }
                        }

                        // Rename path if necessary
                        if let Some(file_name) = file.file_name().map(|f| f.to_string_lossy()) {
                            if file_name.contains(placeholder) {
                                let new_name = file_name.replace(placeholder, value);
                                let new_path = &file_path.with_file_name(&new_name);
                                if file != new_path {
                                    tracing::info!("   ✏️ {} => {}", file.display(), &new_name,);
                                    fs::rename(file_path, new_path).await?;
                                }
                            }
                        }
                    }
                }
            }
            Action::Retain { paths, value } => {
                if *value == Some(false) {
                    // Get files matching
                    let files = omnix_common::fs::find_paths(out_dir).await?;
                    let set = build_glob_set(paths)?;
                    let files_to_delete = files
                        .iter()
                        .filter(|file| set.is_match(file))
                        .collect::<Vec<_>>();
                    if files_to_delete.is_empty() {
                        anyhow::bail!("No paths matched in {:?}", files);
                    };
                    // Iterating in reverse-sorted order ensures that children gets deleted before their parent folders.
                    for file in files_to_delete.iter().sorted().rev() {
                        let path = out_dir.join(file);
                        tracing::info!("   ❌ {}", file.display());
                        omnix_common::fs::remove_all(path).await?;
                    }
                }
            }
        }
        Ok(())
    }
}

// Combine multiple glob patterns into a single set
fn build_glob_set(globs: &[Glob]) -> anyhow::Result<globset::GlobSet> {
    let mut builder = GlobSetBuilder::new();
    for g in globs.iter() {
        builder.add(g.clone());
    }
    Ok(builder.build()?)
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
