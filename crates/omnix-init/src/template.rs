use std::{
    collections::{BTreeMap, HashMap},
    path::{Path, PathBuf},
};

use anyhow::Context;
use itertools::Itertools;
use serde::Deserialize;
use serde_json::Value;

use crate::param;

/// A template in the `om.templates` config
#[derive(Debug, Deserialize, Clone)]
pub struct Template {
    pub template: NixTemplate,
    pub params: Vec<param::Param>,
    #[serde(default)]
    pub tests: BTreeMap<String, super::test::OmInitTest>,
}

/// The official Nix template (`flake.templates.<name>`)
#[derive(Debug, Deserialize, Clone)]
pub struct NixTemplate {
    pub path: PathBuf,
    pub description: Option<String>,
    #[serde(rename = "welcomeText")]
    pub welcome_text: Option<String>,
}

impl Template {
    /// Scaffold the [Template] at the given path.
    //
    /// Returns the canonicalized path of the output directory
    pub async fn scaffold_at(&self, out_dir: &Path) -> anyhow::Result<PathBuf> {
        // Recursively copy the self.template.path to the output directory
        omnix_common::fs::copy_dir_all(&self.template.path, out_dir)
            .await
            .with_context(|| "Unable to copy files")?;

        // Do param replacements
        self.apply_actions(out_dir).await?;

        out_dir
            .canonicalize()
            .with_context(|| "Unable to canonicalize path")
    }

    /// Set 'default' fields of prompts to the user-defined values
    ///
    /// Given a list of prompts, and the user-defined default values for a subset of them (as JSON-parsed `HashMap<String, Value>` where String is the prompt name and serde 'Value' is the 'default' field of action), mutate the prompts to set those 'default' fields
    pub fn set_param_values(&mut self, values: &HashMap<String, Value>) {
        for param in self.params.iter_mut() {
            if let Some(v) = values.get(&param.name) {
                param.set_value(v);
            }
        }
    }

    async fn apply_actions(&self, out_dir: &Path) -> anyhow::Result<()> {
        for param in self.params.iter().sorted_by(|a, b| a.action.cmp(&b.action)) {
            if param.action.has_value() {
                tracing::info!("{}", param);
            }
            param
                .action
                .apply(out_dir.as_ref())
                .await
                .with_context(|| format!("Unable to apply param {}", param.name))?;
        }
        Ok(())
    }
}
