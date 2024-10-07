use std::{collections::HashMap, path::Path};

use crate::command::core::om;
use nix_rs::{command::NixCmd, flake::url::FlakeUrl};
use omnix_init::registry::BUILTIN_REGISTRY;

/// `om init` runs and successfully initializes a template
#[tokio::test]
async fn om_init() -> anyhow::Result<()> {
    for test in om_init_tests() {
        test.run_test().await?;
    }
    Ok(())
}

fn om_init_tests() -> Vec<OmInitTest> {
    let registry = BUILTIN_REGISTRY.clone();
    let lookup = |name: &str| registry.0.get(name).cloned().unwrap();
    vec![
        OmInitTest {
            template_name: lookup("haskell-template"),
            params: r#"{"package-name": "foo", "author": "John", "vscode": false }"#,
            asserts: Asserts {
                source: PathAsserts(HashMap::from([
                    (".github/workflows/ci.yaml", true),
                    (".vscode", false),
                ])),
                packages: HashMap::from([(
                    "default".to_string(),
                    PathAsserts(HashMap::from([("bin/foo", true)])),
                )]),
            },
        },
        OmInitTest {
            template_name: lookup("rust-nix-template"),
            params: r#"{"package-name": "qux", "author": "John", "author-email": "john@example.com" }"#,
            asserts: Asserts {
                source: PathAsserts(HashMap::from([
                    ("Cargo.toml", true),
                    ("flake.nix", true),
                    (".github/workflows/ci.yml", true),
                    (".vscode", true),
                    ("nix/modules/template.nix", false),
                ])),
                packages: HashMap::from([(
                    "default".to_string(),
                    PathAsserts(HashMap::from([("bin/qux", true)])),
                )]),
            },
        },
        OmInitTest {
            template_name: lookup("nixos-unified-template").with_attr("home"),
            params: r#"{"username": "john", "git-email": "jon@ex.com", "git-name": "John", "neovim": true }"#,
            asserts: Asserts {
                source: PathAsserts(HashMap::from([
                    ("modules/home/neovim/default.nix", true),
                    (".github/workflows", false),
                ])),
                packages: HashMap::from([(
                    "homeConfigurations.john.activationPackage".to_string(),
                    PathAsserts(HashMap::from([
                        ("home-path/bin/nvim", true),
                        ("home-path/bin/vim", false),
                    ])),
                )]),
            },
        },
    ]
}

/// A test for a single template
struct OmInitTest {
    /// The template name to pass to `om init`
    template_name: FlakeUrl,
    /// The --default-params to pass to `om init`
    params: &'static str,
    /// Various assertions to make after running `om init`
    asserts: Asserts,
}

impl OmInitTest {
    /// Run this test on a temporary directory
    async fn run_test(&self) -> anyhow::Result<()> {
        let temp_dir = assert_fs::TempDir::new().unwrap();
        om()?
            .args([
                "init",
                "-o",
                &temp_dir.to_string_lossy(),
                &self.template_name,
                "--non-interactive",
                "--params",
                self.params,
            ])
            .assert()
            .success();

        // Recursively print the contents of temp_dir to debug test failures
        let paths = omnix_common::fs::find_paths(&temp_dir).await?;
        println!(
            "[{}] Paths in temp_dir {}:",
            self.template_name,
            temp_dir.path().display()
        );
        for path in paths {
            println!("  {}", path.display());
        }

        // Run assertion tests
        self.asserts.assert(&temp_dir).await?;

        temp_dir.close().unwrap();
        Ok(())
    }
}

#[derive(Default)]
struct Asserts {
    /// [PathAsserts] for the source directory
    source: PathAsserts,
    /// [PathAsserts] for `nix build .#<name>`'s out path
    packages: HashMap<String, PathAsserts>,
}

impl Asserts {
    async fn assert(&self, dir: &Path) -> anyhow::Result<()> {
        self.source.assert(dir);

        for (attr, package) in self.packages.iter() {
            let paths = nix_rs::flake::command::build(
                &NixCmd::default(),
                FlakeUrl::from(dir).with_attr(attr),
            )
            .await?;
            assert_matches!(paths.first().and_then(|v| v.first_output()), Some(path) => {
                package.assert(path);
            });
        }

        Ok(())
    }
}

/// Set of path assertions to make
///
/// If value is true, the path must exist.
#[derive(Default)]
struct PathAsserts(HashMap<&'static str, bool>);

impl PathAsserts {
    fn assert(&self, dir: &Path) {
        for (path, must_exist) in self.0.iter() {
            let check = dir.join(path).exists();
            let verb = if *must_exist { "exist" } else { "not exist" };
            assert!(
                if *must_exist { check } else { !check },
                "Expected path to {}: {:?} (under {:?})",
                verb,
                path,
                dir,
            );
        }
    }
}
