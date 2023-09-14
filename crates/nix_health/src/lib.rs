#![feature(associated_type_defaults)]
//! Health checks for the user's Nix install

pub mod check;
pub mod report;
pub mod traits;

use nix_rs::flake::url::FlakeUrl;
use nix_rs::{env, info};
use serde::{Deserialize, Serialize};

use self::check::{
    caches::Caches, flake_enabled::FlakeEnabled, max_jobs::MaxJobs, min_nix_version::MinNixVersion,
    trusted_users::TrustedUsers,
};
use self::traits::*;

/// Nix Health check information for user's install
///
/// Each field represents an individual check which satisfies the [Checkable] trait.
#[derive(Debug, Default, Serialize, Deserialize, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct NixHealth {
    #[serde(default)]
    pub max_jobs: MaxJobs,
    #[serde(default)]
    pub caches: Caches,
    #[serde(default)]
    pub flake_enabled: FlakeEnabled,
    #[serde(default)]
    pub min_nix_version: MinNixVersion,
    #[serde(default)]
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
    pub fn new(m_flake: Option<FlakeUrl>) -> Self {
        match m_flake {
            None => Self::default(),
            // cf. https://github.com/juspay/nix-browser/issues/60
            Some(_) => unimplemented!("Per-flake health checks are not yet supported"),
        }
    }

    /// Run all checks and collect the results
    pub fn run_checks(&self, nix_info: &info::NixInfo, nix_env: &env::NixEnv) -> Vec<Check> {
        self.into_iter()
            .flat_map(|c| c.check(nix_info, nix_env))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use crate::check::{caches::Caches, min_nix_version::MinNixVersion};

    #[test]
    fn test_json_deserialize_empty() {
        let json = r#"{}"#;
        let v: super::NixHealth = serde_json::from_str(json).unwrap();
        assert_eq!(v.min_nix_version, MinNixVersion::default());
        assert_eq!(v.caches, Caches::default());
        println!("{:?}", v);
    }

    #[test]
    fn test_json_deserialize_some() {
        let json = r#"{ "min-nix-version": { "min-required": "2.17.0" } }"#;
        let v: super::NixHealth = serde_json::from_str(json).unwrap();
        assert_eq!(v.min_nix_version.min_required.to_string(), "2.17.0");
        assert_eq!(v.caches, Caches::default());
    }
}
