use crate::command::core::om;
use assert_cmd::Command;
use assert_fs::prelude::PathChild;
use predicates::str::contains;

/// `om init` runs and successfully initializes a template
#[test]
fn om_init() -> anyhow::Result<()> {
    let temp_dir = assert_fs::TempDir::new().unwrap();

    let template_name = "haskell-template";
    let default_params = r#"{"package-name": "foo", "author": "John", "vscode": false }"#;

    om()?
        .args([
            "init",
            "-o",
            &temp_dir.to_string_lossy(),
            template_name,
            "--non-interactive",
            "--params",
            default_params,
        ])
        .assert()
        .success();

    // File inclusion checks
    // Fail if .vscode/ directory exists in temp_dir
    assert!(!temp_dir.child(".vscode").exists());
    // .github/ must exist (template includes by default)
    assert!(temp_dir.child(".github").exists());
    // .github must have files inside it
    assert!(temp_dir
        .child(".github")
        .child("workflows")
        .child("ci.yaml")
        .exists());

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
