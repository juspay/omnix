use std::fmt::Display;

/// Catch all error to use in UI components
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SystemError {
    pub message: String,
}

impl Display for SystemError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

impl From<String> for SystemError {
    fn from(message: String) -> Self {
        Self { message }
    }
}
