use serde::{Deserialize, Serialize};

/// The (optional) attribute output part of a [super::FlakeUrl]
///
/// Example: `foo` in `.#foo`.
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FlakeAttr(pub Option<String>);

impl FlakeAttr {
    pub fn new(attr: &str) -> Self {
        FlakeAttr(Some(attr.to_owned()))
    }

    pub fn none() -> Self {
        FlakeAttr(None)
    }

    /// Get the attribute name.
    ///
    /// If no such attribute exists, return "default".
    pub fn get_name(&self) -> String {
        self.0.clone().unwrap_or_else(|| "default".to_string())
    }

    /// Whether an explicit attribute is not set
    pub fn is_none(&self) -> bool {
        self.0.is_none()
    }

    /// Return nested attrs if the user specified one is separated by '.'
    pub fn as_list(&self) -> Vec<String> {
        self.0
            .clone()
            .map(|s| s.split('.').map(|s| s.to_string()).collect())
            .unwrap_or_default()
    }
}
