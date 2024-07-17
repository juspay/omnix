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

/// Return the [Command] pointing to the `om` cargo bin
fn om() -> anyhow::Result<Command> {
    Ok(Command::cargo_bin("om")?)
}
