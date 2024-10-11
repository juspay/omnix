use std::{collections::HashMap, path::Path};

use anyhow::Context;
use nix_rs::{
    command::NixCmd,
    flake::{system::System, url::FlakeUrl},
};
use serde::Deserialize;
use serde_json::Value;

use crate::config::FlakeTemplate;

/// A test for a single template
#[derive(Debug, Deserialize, Clone)]
pub struct OmInitTest {
    /// The template name to pass to `om init`
    /// MAKES NO SENSE
    /// template_name: FlakeUrl,
    /// The --default-params to pass to `om init`
    params: HashMap<String, Value>,
    /// Various assertions to make after running `om init`
    asserts: Asserts,
    /// Optional, whitelist of systems to run this test on (others are ignored)
    systems: Option<Vec<System>>,
}

impl OmInitTest {
    /// Can this test be run on this system?
    pub fn can_run_on(&self, system: &System) -> bool {
        if let Some(systems) = self.systems.as_ref() {
            systems.contains(system)
        } else {
            true
        }
    }

    /// Run this test on a temporary directory
    pub async fn run_test<'a>(
        &self,
        url: &FlakeUrl,
        template: &FlakeTemplate<'a>,
    ) -> anyhow::Result<()> {
        let temp_dir = assert_fs::TempDir::new().unwrap();
        let out_dir = temp_dir.path().join("output");
        let mut template = template.clone();

        tracing::info!(
            "🧪 [{:?}] Running test params={:?} systems-whitelist={}",
            &url,
            &self.params,
            self.systems
                .as_ref()
                .map(|systems| systems
                    .iter()
                    .map(|s| s.to_string())
                    .collect::<Vec<_>>()
                    .join(","))
                .unwrap_or_else(|| "all".to_string())
        );
        template.template.set_param_values(&self.params);
        template
            .template
            .scaffold_at(&out_dir)
            .await
            .with_context(|| "Unable to scaffold")?;

        // Recursively print the contents of out_dir to debug test failures
        let paths = omnix_common::fs::find_paths(&out_dir).await?;
        tracing::debug!(
            "Template files (under {}): {}",
            out_dir.display(),
            paths
                .iter()
                .map(|path| path.display().to_string())
                .collect::<Vec<_>>()
                .join("; ")
        );

        // Run assertion tests
        self.asserts.assert(&out_dir).await?;

        temp_dir.close().unwrap();
        Ok(())
    }
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct Asserts {
    /// [PathAsserts] for the source directory
    #[serde(default)]
    source: PathAsserts,

    /// [PathAsserts] for `nix build .#<name>`'s out path
    #[serde(default)]
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
#[derive(Debug, Deserialize, Clone, Default)]
pub struct PathAsserts(HashMap<String, bool>);

impl PathAsserts {
    fn assert(&self, dir: &Path) {
        for (path, must_exist) in self.0.iter() {
            println!("PathAssert {}; exist? ({}) in {:?}", path, must_exist, dir);
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
