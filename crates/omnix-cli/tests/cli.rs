use assert_cmd::Command;
use predicates::prelude::*;

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
    om()?.arg("show").arg(".").assert().success().stdout(
        predicate::str::contains("Packages")
            .and(predicate::str::contains("Devshells"))
            .and(predicate::str::contains("Checks")),
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
        .stdout(
            predicate::str::contains("bar").and(predicate::str::contains(
                "github:srid/haskell-multi-nix/c85563721c388629fa9e538a1d97274861bc8321",
            )),
        );
    Ok(())
}

/// `om init` runs and successfully initializes a template
///
/// NOTE: Test ignored due to https://github.com/mikaelmello/inquire/issues/71
///
/// Alternatively, we can pass all prompt data via CLI args, as JSON. But this
/// may be overkill.
#[ignore]
#[test]
fn om_init() -> anyhow::Result<()> {
    let temp_dir = assert_fs::TempDir::new().unwrap();
    om()?.arg("init").arg(temp_dir.path()).assert().success();
    temp_dir.close().unwrap();
    Ok(())
}

/// Return the [Command] pointing to the `om` cargo bin
fn om() -> anyhow::Result<Command> {
    Ok(Command::cargo_bin("om")?)
}
