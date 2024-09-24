//! Filesystem utilities
use async_walkdir::WalkDir;
use futures_lite::stream::StreamExt;
use std::{
    fs::Permissions,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
};
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

/// Recursively find paths under a directory
///
/// Returned list of files or directories are relative to the given directory.
pub async fn find_paths(dir: impl AsRef<Path> + Copy) -> anyhow::Result<Vec<PathBuf>> {
    let mut paths = Vec::new();
    let mut walker = WalkDir::new(dir);

    while let Some(entry) = walker.next().await {
        let entry = entry?;
        let path = entry.path();
        paths.push(path.strip_prefix(dir)?.to_path_buf());
    }

    Ok(paths)
}

/// Recursively delete the path
pub async fn remove_all(path: impl AsRef<Path>) -> anyhow::Result<()> {
    let path = path.as_ref();
    if path.is_dir() {
        fs::remove_dir_all(path).await?;
    } else {
        fs::remove_file(path).await?;
    }
    Ok(())
}
