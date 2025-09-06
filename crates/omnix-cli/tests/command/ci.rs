use std::path::{Path, PathBuf};

use anyhow::bail;
use nix_rs::store::path::StorePath;
use regex::Regex;
use serde::de::DeserializeOwned;
use serde_json::Value;
use tokio::process::Command;

use super::core::om;

/// Clone a GitHub repository to a temporary directory and run `om ci run` in it
async fn om_ci_run_in_cloned_repo(
    github_url: &str,
    commit_hash: Option<&str>,
    args: &[&str],
) -> anyhow::Result<StorePath> {
    // Create a temporary directory
    let temp_dir = tempfile::tempdir()?;
    let temp_path = temp_dir.path();

    // Clone the repository
    let mut clone_cmd = Command::new("git");
    clone_cmd.args(["clone", github_url, temp_path.to_str().unwrap()]);

    let clone_output = clone_cmd.output().await?;
    if !clone_output.status.success() {
        bail!(
            "Failed to clone repository {}:\n{}",
            github_url,
            String::from_utf8_lossy(&clone_output.stderr)
        );
    }

    // Checkout specific commit if provided
    if let Some(hash) = commit_hash {
        let mut checkout_cmd = Command::new("git");
        checkout_cmd.args(["checkout", hash]);
        checkout_cmd.current_dir(temp_path);

        let checkout_output = checkout_cmd.output().await?;
        if !checkout_output.status.success() {
            bail!(
                "Failed to checkout commit {} in {}:\n{}",
                hash,
                github_url,
                String::from_utf8_lossy(&checkout_output.stderr)
            );
        }
    }

    // Run om ci run in the cloned directory
    let mut cmd = om()?;
    cmd.arg("ci").arg("run").args(args);
    cmd.current_dir(temp_path);

    let output = cmd.output()?;
    if !output.status.success() {
        bail!(
            "Failed to run `om ci run` in {}:\n{}",
            github_url,
            String::from_utf8_lossy(&output.stderr).to_string(),
        );
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    let out = StorePath::new(PathBuf::from(stdout.trim()));
    Ok(out)
}

#[tokio::test]
/// Run `om ci build` and check if the stdout consists of only /nix/store/* paths
async fn build_flake_output() -> anyhow::Result<()> {
    let out = om_ci_run_in_cloned_repo(
        "https://github.com/srid/haskell-multi-nix.git",
        Some("c85563721c388629fa9e538a1d97274861bc8321"),
        &[],
    )
    .await?;

    assert!(
        out.as_path().starts_with("/nix/store/"),
        "Unexpected line in stdout: {}",
        out
    );

    Ok(())
}

#[tokio::test]
/// A simple test, without config
async fn test_haskell_multi_nix() -> anyhow::Result<()> {
    let out = om_ci_run_in_cloned_repo(
        "https://github.com/srid/haskell-multi-nix.git",
        Some("c85563721c388629fa9e538a1d97274861bc8321"),
        &[],
    )
    .await?;
    let v: Value = serde_json::from_reader(std::fs::File::open(&out)?)?;
    let paths: Vec<PathBuf> = lookup_path(&v, &["result", "ROOT", "build", "outPaths"]).unwrap();
    let expected = vec![
        "/nix/store/3x2kpymc1qmd05da20wnmdyam38jkl7s-ghc-shell-for-packages-0",
        "/nix/store/dzhf0i3wi69568m5nvyckck8bbs9yrfd-foo-0.1.0.0",
        "/nix/store/hsj8mwn9vzlyaxzmwyf111scisnjhlkb-bar-0.1.0.0",
        "/nix/store/hsj8mwn9vzlyaxzmwyf111scisnjhlkb-bar-0.1.0.0/bin/bar",
    ]
    .into_iter()
    .map(|s| PathBuf::from(s.to_string()))
    .collect::<Vec<_>>();
    assert_same_drvs(paths, expected);
    Ok(())
}

#[tokio::test]
async fn test_haskell_multi_nix_all_dependencies() -> anyhow::Result<()> {
    let out = om_ci_run_in_cloned_repo(
        "https://github.com/srid/haskell-multi-nix.git",
        Some("c85563721c388629fa9e538a1d97274861bc8321"),
        &["--include-all-dependencies"],
    )
    .await?;
    let v: Value = serde_json::from_reader(std::fs::File::open(&out)?)?;
    let paths: Vec<PathBuf> = lookup_path(&v, &["result", "ROOT", "build", "allDeps"]).unwrap();
    // Since the number of dependencies is huge, we just check for the presence of system-independent
    // source of the `foo` sub-package in `haskell-multi-nix`.
    let expected = PathBuf::from("/nix/store/bpybsny4gd5jnw0lvk5khpq7md6nwg5f-source-foo");
    assert!(paths.contains(&expected));
    Ok(())
}

#[tokio::test]
/// Whether `--override-input` passes CI successfully
async fn test_haskell_multi_nix_override_input() -> anyhow::Result<()> {
    let _out = om_ci_run_in_cloned_repo(
        "https://github.com/srid/haskell-multi-nix.git",
        Some("c85563721c388629fa9e538a1d97274861bc8321"),
        &[
            "--",
            "--override-input",
            "haskell-flake",
            // haskell-flake 0.4 release
            "github:srid/haskell-flake/c8622c8a259e18e0a1919462ce885380108a723c",
        ],
    )
    .await?;
    Ok(())
}

#[tokio::test]
/// A test, with config
async fn test_services_flake() -> anyhow::Result<()> {
    let out = om_ci_run_in_cloned_repo(
        "https://github.com/juspay/services-flake.git",
        Some("23cf162387af041035072ee4a9de20f8408907cb"),
        &["default.simple-example"],
    )
    .await?;
    let v: Value = serde_json::from_reader(std::fs::File::open(&out)?)?;
    let paths: Vec<PathBuf> =
        lookup_path(&v, &["result", "simple-example", "build", "outPaths"]).unwrap();
    let expected = vec![
        "/nix/store/ib83flb2pqjb416qrjbs4pqhifa3hhs4-default-test",
        "/nix/store/l9c8y2xx2iffk8l1ipp4mkval8wl8paa-default",
        "/nix/store/pj2l11lc4kai6av32hgfsrsvmga7vkhf-nix-shell",
    ]
    .into_iter()
    .map(|s| PathBuf::from(s.to_string()))
    .collect::<Vec<_>>();
    assert_same_drvs(paths, expected);
    Ok(())
}

pub fn assert_same_drvs(drvs1: Vec<PathBuf>, drvs2: Vec<PathBuf>) {
    assert_eq!(drvs1.len(), drvs2.len());
    let mut drv1 = drvs1
        .into_iter()
        .map(|d| without_hash(&d))
        .collect::<Vec<_>>();
    let mut drv2 = drvs2
        .into_iter()
        .map(|d| without_hash(&d))
        .collect::<Vec<_>>();
    drv1.sort();
    drv2.sort();
    assert_eq!(drv1, drv2);
}

pub fn without_hash(out_path: &Path) -> String {
    let re = Regex::new(r".+\-(.+)").unwrap();
    let captures = re.captures(out_path.to_str().unwrap()).unwrap();
    captures.get(1).unwrap().as_str().to_string()
}

/// Lookup a path in the [`serde_json::Value`]
fn lookup_path<T>(v: &Value, path: &[&str]) -> Option<T>
where
    T: DeserializeOwned,
{
    match path {
        [] => None,
        [key] => v
            .get(key)
            .and_then(|v| serde_json::from_value(v.clone()).ok()),
        [key, rest @ ..] => v.get(key).and_then(|v| lookup_path(v, rest)),
    }
}
