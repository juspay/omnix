#![feature(associated_type_defaults)]
//! Health checks for the user's Nix install

pub mod check;
pub mod report;
pub mod traits;

use nix_rs::{env, info};

use self::check::{
    caches::Caches, flake_enabled::FlakeEnabled, max_jobs::MaxJobs, min_nix_version::MinNixVersion,
    trusted_users::TrustedUsers,
};
use self::traits::*;

/// Nix Health check information for user's install
pub struct NixHealth(Vec<Box<dyn Checkable>>);

impl Default for NixHealth {
    fn default() -> Self {
        let checks: Vec<Box<dyn Checkable>> = vec![
            // NOTE: UI will use this exact order.
            Box::<MinNixVersion>::default(),
            Box::<FlakeEnabled>::default(),
            Box::<MaxJobs>::default(),
            Box::<Caches>::default(),
            Box::<TrustedUsers>::default(),
        ];
        Self(checks)
    }
}

impl NixHealth {
    /// Run all checks and collect the results
    pub fn run_checks(&self, nix_info: &info::NixInfo, nix_env: &env::NixEnv) -> Vec<Check> {
        self.0
            .iter()
            .flat_map(|c| c.check(nix_info, nix_env))
            .collect()
    }
}
