//! Rust module for Nix flakes
pub mod outputs;
pub mod schema;
#[cfg(feature = "ssr")]
pub mod show;
pub mod system;
pub mod url;

use serde::{Deserialize, Serialize};

use self::{outputs::FlakeOutputs, schema::FlakeSchema, system::System, url::FlakeUrl};

/// All the information about a Nix flake
// #[serde_as]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Flake {
    /// The flake url which this struct represents
    pub url: FlakeUrl,
    /// `nix flake show` output
    pub output: FlakeOutputs,
    /// Flake output schema (typed version of [FlakeOutputs])
    pub schema: FlakeSchema,
    // TODO: Add `nix flake metadata` info.
}
