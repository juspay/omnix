//! Filesystem utilities
use async_walkdir::WalkDir;
use futures_lite::stream::StreamExt;
use std::{fs::Permissions, os::unix::fs::PermissionsExt, path::Path};
use tokio::fs;

/// Copy a directory recursively
///
/// The target directory will always be user readable & writable.
pub async fn copy_dir_all(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> anyhow::Result<()> {
    let src = src.as_ref();
    let dst = dst.as_ref();

    let mut walker = WalkDir::new(src);

    while let Some(entry) = walker.next().await {
        let entry = entry?;
        let path = &entry.path();
        let relative = path.strip_prefix(src)?;
        let target = dst.join(relative);

        if entry.file_type().await?.is_dir() {
            fs::create_dir_all(&target).await?;
            fs::set_permissions(&target, Permissions::from_mode(0o755)).await?;
        } else {
            if let Some(parent) = target.parent() {
                fs::create_dir_all(parent).await?;
                fs::set_permissions(&parent, Permissions::from_mode(0o755)).await?;
            }
            fs::copy(path, &target).await?;
            fs::set_permissions(&target, Permissions::from_mode(0o644)).await?;
        }
    }

    Ok(())
}
