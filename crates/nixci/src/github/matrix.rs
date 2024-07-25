/// Enough types to get branch info from Pull Request URL
use nix_rs::flake::system::System;
use serde::{Deserialize, Serialize};

use crate::config::Subflakes;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GitHubMatrixRow {
    pub system: System,
    pub subflake: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GitHubMatrix {
    pub include: Vec<GitHubMatrixRow>,
}

impl GitHubMatrix {
    pub fn from(systems: Vec<System>, subflakes: &Subflakes) -> Self {
        let include: Vec<GitHubMatrixRow> = systems
            .iter()
            .flat_map(|system| {
                subflakes
                    .0
                    .iter()
                    .filter(|&(_k, v)| v.can_build_on(&[system.clone()]))
                    .map(|(k, _v)| GitHubMatrixRow {
                        system: system.clone(),
                        subflake: k.clone(),
                    })
            })
            .collect();
        GitHubMatrix { include }
    }
}
