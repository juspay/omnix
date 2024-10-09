use std::{collections::HashMap, path::PathBuf, str::FromStr};

use clap::Parser;
use nix_rs::{config::NixConfig, flake::url::FlakeUrl};
use serde_json::Value;

/// Initialize a new flake project
#[derive(Parser, Debug)]
pub struct InitCommand {
    /// Where to create the template
    #[arg(
        name = "OUTPUT_DIR",
        short = 'o',
        long = "output",
        required_unless_present = "test"
    )]
    path: Option<PathBuf>,

    /// The flake from which to initialize the template to use
    ///
    /// Defaults to builtin registry of flake templates.
    #[arg(name = "FLAKE_URL")]
    flake: Option<FlakeUrl>,

    /// Parameter values to use for the template by default.
    #[arg(long = "params")]
    params: Option<Params>,

    /// Whether to disable all prompting, making the command non-interactive
    #[arg(long = "non-interactive")]
    non_interactive: bool,

    /// Run template tests, instead of initializing the template
    #[arg(
        long = "test",
        requires = "FLAKE_URL",
        conflicts_with = "non_interactive",
        conflicts_with = "params",
        conflicts_with = "OUTPUT_DIR"
    )]
    test: bool,
}

impl InitCommand {
    pub async fn run(&self) -> anyhow::Result<()> {
        // Prompt from builtin registry if the user has not specified one.
        let flake = match self.flake {
            Some(ref flake) => flake,
            None => &omnix_init::core::select_from_registry().await?,
        };
        if self.test {
            let cfg = NixConfig::get().await.as_ref()?;
            omnix_init::core::run_tests(&cfg.system.value, flake).await?;
        } else {
            let path = self.path.as_ref().unwrap(); // unwrap is okay, because of `required_unless_present`
            let params = self
                .params
                .as_ref()
                .map_or_else(HashMap::new, |hm| hm.0.clone());
            omnix_init::core::run(path, flake, &params, self.non_interactive).await?;
        }
        Ok(())
    }
}

/// A map of parameter values
#[derive(Clone, Debug, Default)]
struct Params(HashMap<String, Value>);

/// Convenience for passing JSON in command line
impl FromStr for Params {
    type Err = serde_json::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let map: HashMap<String, Value> = serde_json::from_str(s)?;
        Ok(Params(map))
    }
}
