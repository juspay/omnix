//! Github Actions matrix
use nix_rs::flake::system::System;
use serde::{Deserialize, Serialize};

use crate::config::subflakes::SubflakesConfig;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// A row in the Github Actions matrix configuration
pub struct GitHubMatrixRow {
    /// System to build on
    pub system: System,
    /// Subflake to build
    pub subflake: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Github Actions matrix configuration
pub struct GitHubMatrix {
    /// The includes
    pub include: Vec<GitHubMatrixRow>,
}

impl GitHubMatrix {
    /// Create a [GitHubMatrix] for the given subflakes and systems
    pub fn from(systems: Vec<System>, subflakes: &SubflakesConfig) -> Self {
        let include: Vec<GitHubMatrixRow> = systems
            .iter()
            .flat_map(|system| {
                subflakes
                    .0
                    .iter()
                    .filter(|&(_k, v)| v.can_run_on(std::slice::from_ref(system)))
                    .map(|(k, _v)| GitHubMatrixRow {
                        system: system.clone(),
                        subflake: k.clone(),
                    })
            })
            .collect();
        GitHubMatrix { include }
    }
}
