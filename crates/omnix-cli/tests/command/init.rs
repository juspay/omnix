use assert_cmd::Command;
use predicates::str::contains;
use rexpect::spawn;

/// `om init` runs and successfully initializes a template
#[test]
fn om_init() -> anyhow::Result<()> {
    let temp_dir = assert_fs::TempDir::new().unwrap();

    // TOD: back to using this
    // om()?.arg("init").arg(temp_dir.path()).assert().success();
    let om = assert_cmd::cargo::cargo_bin("om");

    let template_name = "haskell-template";
    let default_params = r#"
      '{"package-name": "foo", "author": "John", "vscode": false }'
    "#;

    let mut p = spawn(
        &format!(
            "{:?} init -o {} {} --non-interactive --params {}",
            om,
            temp_dir.path().display(),
            template_name,
            default_params
        ),
        Some(30_000),
    )?;
    p.exp_eof()?;

    // Run the generated template, and compare output.
    // Is there a better way of doing these checks? Property tests + ?
    // TODO: github token?
    Command::new("nix")
        .arg("run")
        .arg(format!("path:{}#foo", &temp_dir.path().display()))
        .assert()
        .success()
        .stdout(contains("from foo"));

    temp_dir.close().unwrap();
    Ok(())
}
