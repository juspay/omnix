use assert_cmd::Command;
use predicates::str::contains;
use rexpect::spawn;

/// `om init` runs and successfully initializes a template
#[test]
fn om_init() -> anyhow::Result<()> {
    let temp_dir = assert_fs::TempDir::new().unwrap();

    // We can't use `om()`; see https://github.com/mikaelmello/inquire/issues/71
    // om()?.arg("init").arg(temp_dir.path()).assert().success();
    let om = assert_cmd::cargo::cargo_bin("om");

    let mut p = spawn(
        &format!("{:?} init {}", om, temp_dir.path().display()),
        Some(30_000),
    )?;
    p.exp_string("Select a template")?;
    p.send_line("haskell-template")?;
    p.exp_string("Author")?;
    p.send_line("")?;
    p.exp_string("Package Name")?;
    p.send_line("foo")?;
    p.exp_string("VSCode support")?;
    p.send_line("")?;
    p.exp_string("Nix Template")?;
    p.send_line("")?;
    p.exp_string("GitHub Actions")?;
    p.send_line("")?;
    p.exp_eof()?;

    // Run the generated template, and compare output.
    // Is there a better way of doing these checks? Property tests + ?
    // TODO: github token?
    Command::new("nix")
        .arg("run")
        .arg(format!("path:{}#foo", &temp_dir.path().display()))
        .assert()
        .success()
        .stdout(contains("Hello"));

    temp_dir.close().unwrap();
    Ok(())
}
