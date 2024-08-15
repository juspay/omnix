use nix_rs::flake::url::FlakeUrl;

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
