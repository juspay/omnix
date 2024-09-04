use std::{collections::HashMap, path::PathBuf, str::FromStr};

use clap::Parser;
use serde_json::Value;

/// Initialize a new flake project
#[derive(Parser, Debug)]
pub struct InitCommand {
    /// Where to create the template
    #[arg(short = 'o', long = "output")]
    path: PathBuf,

    /// The name of the template to use
    ///
    /// If not passed, the user will presented with a list of templates to choose from.
    ///
    /// In future, this will support arbitrary URLs. For now, we only support builtin templates.
    template: Option<String>,

    /// Parameter values to use for the template by default.
    #[arg(long = "params")]
    params: Params,
}

impl InitCommand {
    pub async fn run(&self) -> anyhow::Result<()> {
        tracing::warn!("\n  !! WARNING: `om init` is still under development !!\n");
        omnix_init::core::initialize_template(&self.path, self.template.clone(), &self.params.0)
            .await
    }
}

/// A map of parameter values
#[derive(Clone, Debug)]
struct Params(HashMap<String, Value>);

/// Convenience for passing JSON in command line
impl FromStr for Params {
    type Err = serde_json::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let map: HashMap<String, Value> = serde_json::from_str(s)?;
        Ok(Params(map))
    }
}
