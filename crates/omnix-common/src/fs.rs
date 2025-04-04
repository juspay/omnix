//! Filesystem utilities
use async_walkdir::{DirEntry, WalkDir};
use futures_lite::stream::StreamExt;
use std::{
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
};
use tokio::fs;

/// Copy a directory recursively
///
/// The target directory will always be user readable & writable.
pub async fn copy_dir_all(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> anyhow::Result<()> {
    let mut walker = WalkDir::new(&src);

    while let Some(entry) = walker.next().await {
        copy_entry(&src, entry?, &dst).await?;
    }
    Ok(())
}

async fn copy_entry(
    src: impl AsRef<Path>,
    entry: DirEntry,
    dst: impl AsRef<Path>,
) -> anyhow::Result<()> {
    let path = &entry.path();
    let relative = path.strip_prefix(src)?;
    let target = dst.as_ref().join(relative);

    let file_type = entry.file_type().await?;
    if file_type.is_dir() {
        // Handle directories
        fs::create_dir_all(&target).await?;
    } else {
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent).await?;
        }
        if file_type.is_symlink() {
            // Handle symlinks *as is* (we expect relative symlink targets) without resolution.
            let link_target = fs::read_link(path).await?;
            fs::symlink(&link_target, &target).await?;
        } else {
            // Handle regular files
            fs::copy(path, &target).await?;
            // Because we are copying from the Nix store, the source paths will be read-only.
            // So, make the target writeable by the owner.
            make_owner_writeable(&target).await?;
        }
    }
    Ok(())
}

async fn make_owner_writeable(path: impl AsRef<Path>) -> anyhow::Result<()> {
    let path = path.as_ref();
    let mut perms = fs::metadata(path).await?.permissions();
    perms.set_mode(perms.mode() | 0o600); // Read/write for owner
    fs::set_permissions(path, perms).await?;
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
