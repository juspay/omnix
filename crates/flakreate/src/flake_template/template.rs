use colored::Colorize;
use core::fmt;
use std::fmt::{Display, Formatter};

use serde::{Deserialize, Serialize};

use super::{config::FlakeTemplateConfig, fileop::FileOp};

/// A Nix flake template
///
/// Defined per [this definition](https://nix.dev/manual/nix/2.22/command-ref/new-cli/nix3-flake-init#template-definitions) in the flake.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FlakeTemplate {
    #[serde(skip_deserializing)]
    pub name: String,

    pub description: String,

    pub path: String,

    #[serde(rename = "welcomeText")]
    pub welcome_text: Option<String>,

    #[serde(flatten)]
    pub config: FlakeTemplateConfig,
}

impl Display for FlakeTemplate {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if self.config.tags.is_empty() {
            write!(f, "{}", self.name)
        } else {
            write!(
                f,
                "{:<20} {}",
                self.name,
                self.config.tags.join(", ").dimmed()
            )
        }
    }
}

impl FlakeTemplate {
    /// Polyfill unserialized fields
    pub fn polyfill(&mut self, name: String) {
        self.name = name;
    }

    pub fn prompt_replacements(&self) -> anyhow::Result<Vec<Vec<FileOp>>> {
        self.config
            .params
            .iter()
            .map(|param| param.prompt_value())
            .collect()
    }
}
