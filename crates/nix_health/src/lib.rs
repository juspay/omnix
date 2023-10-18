//! Health checks for the user's Nix install

pub mod check;
pub mod report;
pub mod traits;

use check::direnv::Direnv;
use serde::{Deserialize, Serialize};
use tracing::instrument;

use self::check::{
    caches::Caches, flake_enabled::FlakeEnabled, max_jobs::MaxJobs, min_nix_version::MinNixVersion,
    rosetta::Rosetta, trusted_users::TrustedUsers,
};

/// Nix Health check information for user's install
///
/// Each field represents an individual check which satisfies the [Checkable] trait.
#[derive(Debug, Default, Serialize, Deserialize, Clone)]
#[serde(default, rename_all = "kebab-case")]
pub struct NixHealth {
    pub max_jobs: MaxJobs,
    pub caches: Caches,
    pub flake_enabled: FlakeEnabled,
    pub nix_version: MinNixVersion,
    pub system: check::system::System,
    pub trusted_users: TrustedUsers,
    pub rosetta: Rosetta,
    pub direnv: Direnv,
}

impl<'a> IntoIterator for &'a NixHealth {
    type Item = &'a dyn traits::Checkable;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    /// Return an iterator to iterate on the fields of [NixHealth]
    fn into_iter(self) -> Self::IntoIter {
        let items: Vec<Self::Item> = vec![
            &self.rosetta,
            &self.nix_version,
            &self.flake_enabled,
            &self.system,
            &self.max_jobs,
            &self.caches,
            &self.trusted_users,
            &self.direnv,
        ];
        items.into_iter()
    }
}

impl NixHealth {
    /// Create [NixHealth] using configuration from the given flake
    ///
    /// Fallback to using the default health check config if the flake doesn't
    /// override it.
    pub async fn from_flake(
        url: nix_rs::flake::url::FlakeUrl,
    ) -> Result<Self, nix_rs::command::NixCmdError> {
        use nix_rs::flake::eval::nix_eval_attr_json;
        nix_eval_attr_json(&url).await
    }

    /// Run all checks and collect the results
    #[instrument(skip_all)]
    pub fn run_checks(
        &self,
        nix_info: &nix_rs::info::NixInfo,
        nix_env: &nix_rs::env::NixEnv,
    ) -> Vec<traits::Check> {
        tracing::info!("🩺 Running health checks");
        self.into_iter()
            .flat_map(|c| c.check(nix_info, nix_env))
            .collect()
    }

    pub fn schema() -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(&NixHealth::default())
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
    fn test_json_deserialize_nix_version() {
        let json = r#"{ "nix-version": { "min-required": "2.17.0" } }"#;
        let v: super::NixHealth = serde_json::from_str(json).unwrap();
        assert_eq!(v.nix_version.min_required.to_string(), "2.17.0");
        assert_eq!(v.caches, Caches::default());
    }

    #[test]
    fn test_json_deserialize_caches() {
        let json = r#"{ "caches": { "required": ["https://foo.cachix.org"] } }"#;
        let v: super::NixHealth = serde_json::from_str(json).unwrap();
        assert_eq!(
            v.caches.required,
            vec![url::Url::parse("https://foo.cachix.org").unwrap()]
        );
        assert_eq!(v.nix_version, MinNixVersion::default());
    }
}
