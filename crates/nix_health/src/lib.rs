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
    pub nix_version: MinNixVersion,
    #[serde(default)]
    pub trusted_users: TrustedUsers,
}

impl<'a> IntoIterator for &'a NixHealth {
    type Item = &'a dyn Checkable;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    /// Return an iterator to iterate on the fields of [NixHealth]
    fn into_iter(self) -> Self::IntoIter {
        let items: Vec<Self::Item> = vec![
            &self.nix_version,
            &self.flake_enabled,
            &self.max_jobs,
            &self.caches,
            &self.trusted_users,
        ];
        items.into_iter()
    }
}

impl NixHealth {
    /// Create [NixHealth] using configuration from the given flake
    ///
    /// Fallback to using the default health check config if the flake doesn't
    /// override it.
    #[cfg(feature = "ssr")]
    pub async fn from_flake(
        url: nix_rs::flake::url::FlakeUrl,
    ) -> Result<Self, nix_rs::command::NixCmdError> {
        use nix_rs::flake::eval::nix_eval_attr_json;
        nix_eval_attr_json(url).await
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
        assert_eq!(v.nix_version, MinNixVersion::default());
        assert_eq!(v.caches, Caches::default());
        println!("{:?}", v);
    }

    #[test]
    fn test_json_deserialize_some() {
        let json = r#"{ "nix-version": { "min-required": "2.17.0" } }"#;
        let v: super::NixHealth = serde_json::from_str(json).unwrap();
        assert_eq!(v.nix_version.min_required.to_string(), "2.17.0");
        assert_eq!(v.caches, Caches::default());
    }
}
