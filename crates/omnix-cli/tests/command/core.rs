use assert_cmd::Command;

/// `om --help` works
#[test]
fn om_help() -> anyhow::Result<()> {
    om()?.arg("--help").assert().success();
    Ok(())
}

/// Return the [Command] pointing to the `om` cargo bin
pub(crate) fn om() -> anyhow::Result<Command> {
    Ok(Command::cargo_bin("om")?)
}
