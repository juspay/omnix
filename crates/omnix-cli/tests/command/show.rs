use predicates::{prelude::*, str::contains};

use super::core::om;

/// `om show` runs, and succeeds for a local flake.
#[test]
fn om_show_local() -> anyhow::Result<()> {
    om()?.arg("show").arg("../..").assert().success().stdout(
        contains("Packages")
            .and(contains("Apps"))
            .and(contains("Devshells"))
            .and(contains("Checks")),
    );
    Ok(())
}

/// `om show` runs, and succeeds for a remote flake.
#[test]
fn om_show_remote() -> anyhow::Result<()> {
    om()?
        .arg("show")
        .arg("github:srid/haskell-multi-nix/c85563721c388629fa9e538a1d97274861bc8321")
        .assert()
        .success()
        .stdout(contains("bar").and(contains(
            "github:srid/haskell-multi-nix/c85563721c388629fa9e538a1d97274861bc8321",
        )));
    Ok(())
}
