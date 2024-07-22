use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// An operation on a file part of the flake template
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum FileOp {
    /// Replace all occurrences of `from` with `to` in the file content
    ContentReplace(PathBuf, String, String),
    /// Rename the file to the given name
    FileRename(PathBuf, String),
    /// Delete the file or directory
    PathDelete(PathBuf),
}

impl FileOp {
    /// Return the list of [`FileOp`]s required to replace the placeholder
    /// `from` to an user-provided `to` across the given set of files.
    ///
    /// The placeholder is expected to be in both the file content and the file
    /// name.
    pub fn ops_for_replacing(from: &str, to: &str, files: &[PathBuf]) -> Vec<FileOp> {
        files
            .iter()
            .flat_map(|file| {
                let mut items: Vec<FileOp> = vec![];
                if to != from {
                    items.push(FileOp::ContentReplace(
                        file.clone(),
                        from.to_string(),
                        to.to_string(),
                    ));
                    if file.to_string_lossy().contains(from) {
                        items.push(FileOp::FileRename(
                            file.clone(),
                            file.to_string_lossy().replace(from, to),
                        ))
                    }
                }
                items
            })
            .collect()
    }

    pub async fn apply(ops: &[Self]) -> anyhow::Result<()> {
        // TODO: Refactor the LLM generated code below
        // TODO: Append thesae paths to base dir
        for op in ops {
            match op {
                FileOp::ContentReplace(file, from, to) => {
                    let content = tokio::fs::read_to_string(file).await?;
                    let content = content.replace(from, to);
                    println!("replace: {} : {} -> {}", file.display(), from, to);
                    tokio::fs::write(file, content).await?;
                }
                FileOp::FileRename(file, new_name) => {
                    println!("rename: {} -> {}", file.display(), new_name);
                    tokio::fs::rename(file, new_name).await?;
                }
                FileOp::PathDelete(path) => {
                    println!("delete: {}", path.display());
                    // FIXME: Careful not to delete anything outside of base dir!
                    tokio::fs::remove_dir_all(path).await?;
                }
            }
        }
        Ok(())
    }
}
