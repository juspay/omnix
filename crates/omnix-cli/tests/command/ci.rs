use std::path::{Path, PathBuf};

use anyhow::bail;
use nix_rs::store::path::StorePath;
use regex::Regex;

use super::core::om;

/// Run `om ci run` passing given arguments, returning its stdout (parsed).
async fn om_ci_run(args: &[&str]) -> anyhow::Result<Vec<StorePath>> {
    let mut cmd = om()?;
    cmd.arg("ci").arg("run").args(args);

    let output = cmd.output()?;
    if !output.status.success() {
        bail!(
            "Failed to run `om ci run`:\n{}",
            String::from_utf8_lossy(&output.stderr).to_string(),
        );
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    let lines = stdout.lines();
    let outs = lines
        .map(|line| StorePath::new(PathBuf::from(line)))
        .collect();
    Ok(outs)
}

#[tokio::test]
/// Run `om ci build` and check if the stdout consists of only /nix/store/* paths
async fn build_flake_output() -> anyhow::Result<()> {
    let outs =
        om_ci_run(&["github:srid/haskell-multi-nix/c85563721c388629fa9e538a1d97274861bc8321"])
            .await?;

    for out in outs {
        assert!(
            out.as_path().starts_with("/nix/store/"),
            "Unexpected line in stdout: {}",
            out
        );
    }

    Ok(())
}

#[tokio::test]
/// A simple test, without config
async fn test_haskell_multi_nix() -> anyhow::Result<()> {
    let outs =
        om_ci_run(&["github:srid/haskell-multi-nix/c85563721c388629fa9e538a1d97274861bc8321"])
            .await?;
    let drv_outs: Vec<PathBuf> = outs
        .into_iter()
        .filter_map(|drv_result| {
            if let StorePath::Other(drv_out) = drv_result {
                Some(drv_out)
            } else {
                None
            }
        })
        .collect();
    let expected = vec![
        "/nix/store/3x2kpymc1qmd05da20wnmdyam38jkl7s-ghc-shell-for-packages-0",
        "/nix/store/dzhf0i3wi69568m5nvyckck8bbs9yrfd-foo-0.1.0.0",
        "/nix/store/hsj8mwn9vzlyaxzmwyf111scisnjhlkb-bar-0.1.0.0",
        "/nix/store/hsj8mwn9vzlyaxzmwyf111scisnjhlkb-bar-0.1.0.0/bin/bar",
    ]
    .into_iter()
    .map(|s| PathBuf::from(s.to_string()))
    .collect::<Vec<_>>();
    assert_same_drvs(drv_outs, expected);
    Ok(())
}

#[tokio::test]
async fn test_haskell_multi_nix_all_dependencies() -> anyhow::Result<()> {
    let outs = om_ci_run(&[
        "--print-all-dependencies",
        "github:srid/haskell-multi-nix/c85563721c388629fa9e538a1d97274861bc8321",
    ])
    .await?;
    // Since the number of dependencies is huge, we just check for the presence of system-independent
    // source of the `foo` sub-package in `haskell-multi-nix`.
    let expected = StorePath::Other(PathBuf::from(
        "/nix/store/bpybsny4gd5jnw0lvk5khpq7md6nwg5f-source-foo",
    ));
    assert!(outs.contains(&expected));
    Ok(())
}

#[tokio::test]
/// A test, with config
async fn test_services_flake() -> anyhow::Result<()> {
    let outs = om_ci_run(&[
        "github:juspay/services-flake/23cf162387af041035072ee4a9de20f8408907cb#default.simple-example",
    ])
    .await?;
    let drv_outs: Vec<PathBuf> = outs
        .into_iter()
        .filter_map(|drv_result| {
            if let StorePath::Other(drv_out) = drv_result {
                Some(drv_out)
            } else {
                None
            }
        })
        .collect();
    let expected = vec![
        "/nix/store/ib83flb2pqjb416qrjbs4pqhifa3hhs4-default-test",
        "/nix/store/l9c8y2xx2iffk8l1ipp4mkval8wl8paa-default",
        "/nix/store/pj2l11lc4kai6av32hgfsrsvmga7vkhf-nix-shell"
    ]
    .into_iter()
    .map(|s| PathBuf::from(s.to_string()))
    .collect::<Vec<_>>();
    assert_same_drvs(drv_outs, expected);
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
