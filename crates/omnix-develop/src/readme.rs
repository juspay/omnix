use serde::Deserialize;

const DEFAULT: &str = r#"🍾 Welcome to the project

*(Want to show custom instructions here? Add them to the `om.develop.default.readme` field in your `flake.nix` or `om.yaml` file)*
"#;

/// The README to display at the end.
#[derive(Debug, Deserialize, Clone)]
pub struct Readme(pub String);

impl Default for Readme {
    fn default() -> Self {
        Self(DEFAULT.to_string())
    }
}

impl Readme {
    /// Get the Markdown string
    pub fn get_markdown(&self) -> &str {
        &self.0
    }
}
