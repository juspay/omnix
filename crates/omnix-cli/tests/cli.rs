use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::process::Command;

/// `om --help` works
#[test]
fn om_help() -> anyhow::Result<()> {
    om()?.arg("--help").assert().success();
    Ok(())
}

/// `om health` runs, and succeeds.
#[test]
fn om_health() -> anyhow::Result<()> {
    om()?.arg("health").assert().success().stdout(
        predicate::str::contains("All checks passed")
            .or(predicate::str::contains("Required checks passed")),
    );
    Ok(())
}

/// `om show` runs, and succeeds for a local flake.
#[test]
fn om_show_local() -> anyhow::Result<()> {
    om()?.arg("show").arg(".").assert().success();
    Ok(())
}

/// `om show` runs, and succeeds for a remote flake.
#[test]
fn om_show_remote() -> anyhow::Result<()> {
    om()?.arg("show").arg("github:srid/haskell-multi-nix/c85563721c388629fa9e538a1d97274861bc8321").assert().success();
    Ok(())
}

/// Return the [Command] pointing to the `om` cargo bin
fn om() -> anyhow::Result<Command> {
    Ok(Command::cargo_bin("om")?)
}
