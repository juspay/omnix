use crate::command::core::om;
use assert_cmd::Command;
use assert_fs::prelude::PathChild;
use predicates::str::contains;

/// `om init` runs and successfully initializes a template
#[test]
fn om_init() -> anyhow::Result<()> {
    for test in om_init_tests() {
        test.run_test()?;
    }
    Ok(())
}

fn om_init_tests() -> Vec<OmInitTest> {
    vec![
        OmInitTest {
            template_name: "haskell-template",
            default_params: r#"{"package-name": "foo", "author": "John", "vscode": false }"#,
            assert_path_exists: vec![".github/workflows/ci.yaml"],
            assert_path_not_exists: vec![".vscode"],
            assert_nix_run_output_contains: "from foo",
        },
        OmInitTest {
            template_name: "rust-nix-template",
            default_params: r#"{"package-name": "qux", "author": "John", "author-email": "john@example.com" }"#,
            assert_path_exists: vec![
                "Cargo.toml",
                "flake.nix",
                ".github/workflows/ci.yml",
                ".vscode",
            ],
            assert_path_not_exists: vec!["nix/modules/template.nix"],
            assert_nix_run_output_contains: "from qux",
        },
    ]
}

/// A test for a single template
struct OmInitTest {
    /// The template name to pass to `om init`
    template_name: &'static str,
    /// The --default-params to pass to `om init`
    default_params: &'static str,
    /// These paths must exist in the output directory
    assert_path_exists: Vec<&'static str>,
    /// These paths must not exist
    assert_path_not_exists: Vec<&'static str>,
    /// `nix run` should produce a certain output (contains this string)
    assert_nix_run_output_contains: &'static str,
}

impl OmInitTest {
    /// Run this test on a temporary directory
    fn run_test(&self) -> anyhow::Result<()> {
        let temp_dir = assert_fs::TempDir::new().unwrap();
        om()?
            .args([
                "init",
                "-o",
                &temp_dir.to_string_lossy(),
                self.template_name,
                "--non-interactive",
                "--params",
                self.default_params,
            ])
            .assert()
            .success();

        for path in &self.assert_path_exists {
            assert!(
                temp_dir.child(path).exists(),
                "Expected path to exist: {}",
                path
            );
        }
        for path in &self.assert_path_not_exists {
            assert!(
                !temp_dir.child(path).exists(),
                "Expected path to not exist: {}",
                path
            );
        }

        // Run the generated template, and compare output.
        // TODO: github token?
        Command::new("nix")
            .arg("run")
            .arg(format!("path:{}", &temp_dir.path().display()))
            .assert()
            .success()
            .stdout(contains(self.assert_nix_run_output_contains));
        temp_dir.close().unwrap();
        Ok(())
    }
}
