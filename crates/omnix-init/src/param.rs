use std::fmt::{self, Display, Formatter};

use serde::Deserialize;
use serde_json::Value;

use crate::action::Action;

/// A template parameter that allows dynamically initializing it.
#[derive(Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Param {
    pub name: String,
    pub description: String,
    #[serde(flatten)]
    pub action: Action,
}

impl Display for Param {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "🪃 {} {}", self.name, self.action)
    }
}

impl Param {
    pub fn set_value(&mut self, val: &Value) {
        match &mut self.action {
            Action::Replace { value, .. } => {
                *value = val.as_str().map(|s| s.to_string());
            }
            Action::Retain { value, .. } => {
                *value = val.as_bool();
            }
        }
    }

    /// Prompt the user for a value for this [Param] using inquire crate.
    pub fn set_value_by_prompting(&mut self) -> anyhow::Result<()> {
        match &mut self.action {
            Action::Replace { placeholder, value } => {
                let mut p = inquire::Text::new(&self.description).with_placeholder(placeholder);
                if let Some(def) = value.as_ref() {
                    p = p.with_default(def);
                }

                let to = p.prompt()?;
                if !to.is_empty() {
                    *value = Some(to);
                }
            }
            Action::Retain { paths: _, value } => {
                let mut p = inquire::Confirm::new(&self.description);
                if let Some(def) = value {
                    p = p.with_default(*def);
                }

                let v = p.prompt()?;
                *value = Some(v)
            }
        }
        Ok(())
    }
}
