use std::path::PathBuf;

use inquire::{Confirm, Text};
use serde::{Deserialize, Serialize};

use super::fileop::FileOp;

/// A parameter to be filled in by the user in a nix flake template path.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Param {
    /// Main message when prompting the user for input
    name: String,
    /// Message displayed at the line below the prompt.
    help: String,
    /// The default value used in the template files, that must be replaced by
    /// the user provided value (if it is different)
    default: Val,
    /// Short hint that describes the expected value of the input.
    placeholder: Option<String>,
    /// Files to do replacement on.
    files: Vec<PathBuf>,
    /// Whether the user must provide a value
    #[serde(default)]
    required: bool,
}

/// The `Param` type.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(untagged)]
enum Val {
    Str(String),
    Bool(bool),
}

impl Param {
    /// Prompt the user for a value for this parameter.
    ///
    /// Return a [`FileOp`] that knows how to apply the replacement.
    pub fn prompt_value(&self) -> anyhow::Result<Vec<FileOp>> {
        match self.default.clone() {
            Val::Str(v) => self.prompt_str(v),
            Val::Bool(v) => self.prompt_bool(v),
        }
    }
    pub fn prompt_bool(&self, default: bool) -> anyhow::Result<Vec<FileOp>> {
        let v = Confirm::new(&self.name)
            .with_help_message(&self.help)
            .with_default(default)
            .prompt()?;
        if v {
            // We expect the template to include all these conditional files by
            // default, so there's nothing to do.
            Ok(vec![])
        } else {
            let ops = self
                .files
                .iter()
                .map(|file| FileOp::PathDelete(file.clone()))
                .collect();
            Ok(ops)
        }
    }
    pub fn prompt_str(&self, default: String) -> anyhow::Result<Vec<FileOp>> {
        let to = Text::new(&self.name)
            .with_help_message(&self.help)
            .with_placeholder(self.placeholder.as_deref().unwrap_or(""))
            .with_default(&default)
            .prompt()?;
        let from = default.clone();
        let ops = FileOp::ops_for_replacing(&from, &to, &self.files);
        // TODO: return nothing if from == to
        Ok(ops)
    }
}
