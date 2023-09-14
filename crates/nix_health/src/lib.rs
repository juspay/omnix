#![feature(associated_type_defaults)]
//! Health checks for the user's Nix install

pub mod check;
pub mod report;
pub mod traits;

use nix_rs::{env, info};
use serde::{Deserialize, Serialize};

use self::check::{
    caches::Caches, flake_enabled::FlakeEnabled, max_jobs::MaxJobs, min_nix_version::MinNixVersion,
    trusted_users::TrustedUsers,
};
use self::traits::*;

/// Nix Health check information for user's install
///
/// Each field represents an individual check which satisfies the [Check] trait.
///
/// NOTE: This struct is isomorphic to [Vec<Box<&dyn Check>>]. We cannot use the
/// latter due to (wasm) serialization limitation with dyn trait objects. An
// [IntoIterator] impl is provide towards this end.
#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct NixHealth {
    pub max_jobs: MaxJobs,
    pub caches: Caches,
    pub flake_enabled: FlakeEnabled,
    pub min_nix_version: MinNixVersion,
    pub trusted_users: TrustedUsers,
}

impl<'a> IntoIterator for &'a NixHealth {
    type Item = &'a dyn Checkable;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    /// Return an iterator to iterate on the fields of [NixHealth]
    fn into_iter(self) -> Self::IntoIter {
        let items: Vec<Self::Item> = vec![
            &self.min_nix_version,
            &self.flake_enabled,
            &self.max_jobs,
            &self.caches,
            &self.trusted_users,
        ];
        items.into_iter()
    }
}

impl NixHealth {
    /// Run all checks and collect the results
    pub fn run_checks(&self, nix_info: &info::NixInfo, nix_env: &env::NixEnv) -> Vec<Check> {
        self.into_iter()
            .flat_map(|c| c.check(nix_info, nix_env))
            .collect()
    }
}
