//! A reference to some flake living somewhere
use std::{
    fmt::{Display, Formatter},
    str::FromStr,
};

use anyhow::Result;
use nix_rs::flake::url::FlakeUrl;

use crate::github::pull_request::{PullRequest, PullRequestRef};

/// A reference to some flake living somewhere
///
/// This type captures the superset of what flake URLs allow.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FlakeRef {
    /// A github PR
    GithubPR(PullRequestRef),
    /// A flake URL supported by Nix commands
    Flake(FlakeUrl),
}

impl FromStr for FlakeRef {
    type Err = String;
    fn from_str(s: &str) -> std::result::Result<FlakeRef, String> {
        let flake_ref = match PullRequestRef::from_web_url(s) {
            Some(pr) => FlakeRef::GithubPR(pr),
            None => FlakeRef::Flake(FlakeUrl(s.to_string())),
        };
        Ok(flake_ref)
    }
}

impl Display for FlakeRef {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            FlakeRef::GithubPR(pr) => write!(f, "{}", pr),
            FlakeRef::Flake(url) => write!(f, "{}", url),
        }
    }
}

impl FlakeRef {
    /// Convert the value to a flake URL that Nix command will recognize.
    pub async fn to_flake_url(&self) -> Result<FlakeUrl> {
        match self {
            FlakeRef::GithubPR(pr) => {
                let pr = PullRequest::get(pr).await?;
                Ok(pr.flake_url())
            }
            FlakeRef::Flake(url) => Ok(url.clone()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_github_pr() {
        assert_eq!(
            FlakeRef::from_str("https://github.com/srid/nixci/pull/19").unwrap(),
            FlakeRef::GithubPR(PullRequestRef {
                owner: "srid".to_string(),
                repo: "nixci".to_string(),
                pr: 19
            })
        );
    }

    #[test]
    fn test_current_dir() {
        assert_eq!(
            FlakeRef::from_str(".").unwrap(),
            FlakeRef::Flake(FlakeUrl(".".to_string()))
        );
    }

    #[test]
    fn test_flake_url() {
        assert_eq!(
            FlakeRef::from_str("github:srid/nixci").unwrap(),
            FlakeRef::Flake(FlakeUrl("github:srid/nixci".to_string()))
        );
    }
}
