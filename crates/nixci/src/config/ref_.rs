//! A reference to a nixci configuration
use nix_rs::flake::url::{attr::FlakeAttr, FlakeUrl};

/// A reference into one or all [crate::config::subflakes::SubflakesConfig] of some [FlakeUrl]
#[derive(Debug)]
pub struct ConfigRef {
    /// The flake itself
    pub flake_url: FlakeUrl,

    /// The name of the nixci configuration (`omci.<name>`) selected
    pub selected_name: String,

    /// The selected sub-flake name if any.
    pub selected_subflake: Option<String>,
}

impl ConfigRef {
    /// Return the non-default attribute that selected this configuration.
    pub fn get_attr(&self) -> FlakeAttr {
        if let Some(subflake) = &self.selected_subflake {
            FlakeAttr::new(&format!("{}.{}", self.selected_name, subflake))
        } else if self.selected_name == "default" {
            FlakeAttr::none()
        } else {
            FlakeAttr::new(&self.selected_name)
        }
    }
}
