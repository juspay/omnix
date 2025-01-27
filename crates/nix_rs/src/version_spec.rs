//! Version requirement spec for [NixVersion]

use std::{fmt, str::FromStr};

use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_with::DeserializeFromStr;
use thiserror::Error;

use crate::version::NixVersion;

/// An individual component of [NixVersionReq]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum NixVersionSpec {
    /// Version must be greater than the specified version
    Gt(NixVersion),
    /// Version must be greater than or equal to the specified version
    Gteq(NixVersion),
    /// Version must be less than the specified version
    Lt(NixVersion),
    /// Version must be less than or equal to the specified version
    Lteq(NixVersion),
    /// Version must not equal the specified version
    Neq(NixVersion),
}

/// Version requirement for [NixVersion]
///
/// Example:
/// ">=2.8, <2.14, 12.13.4"
#[derive(Debug, Clone, PartialEq, Serialize, DeserializeFromStr)]
pub struct NixVersionReq {
    /// List of version specifications
    pub specs: Vec<NixVersionSpec>,
}

/// Errors that can occur while parsing or validating version specifications
#[derive(Error, Debug)]
pub enum BadNixVersionSpec {
    /// Regex error
    #[error("Regex error: {0}")]
    Regex(#[from] regex::Error),

    /// Invalid [NixVersionSpec]
    #[error("Parse error(regex): Invalid version spec format")]
    InvalidFormat,

    /// Parse error (Int)
    #[error("Parse error(int): Invalid version spec format")]
    Parse(#[from] std::num::ParseIntError),

    /// An unknown comparison operator was used
    #[error("Unknown operator in the Nix version spec: {0}")]
    UnknownOperator(String),
}

impl FromStr for NixVersionReq {
    type Err = BadNixVersionSpec;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let specs = s
            .split(',')
            .map(str::trim)
            .map(NixVersionSpec::from_str)
            .collect::<Result<Vec<_>, _>>()?;

        Ok(NixVersionReq { specs })
    }
}

impl NixVersionSpec {
    /// Checks if a given Nix version satisfies this version specification
    pub fn matches(&self, version: &NixVersion) -> bool {
        match self {
            NixVersionSpec::Gt(v) => version > v,
            NixVersionSpec::Gteq(v) => version >= v,
            NixVersionSpec::Lt(v) => version < v,
            NixVersionSpec::Lteq(v) => version <= v,
            NixVersionSpec::Neq(v) => version != v,
        }
    }
}

impl FromStr for NixVersionSpec {
    type Err = BadNixVersionSpec;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use NixVersionSpec::{Gt, Gteq, Lt, Lteq, Neq};
        let re = Regex::new(
            r#"(?x)
            ^
            (?P<op>>=|<=|>|<|!=)
            (?P<major>\d+)
            (?:\.
                (?P<minor>\d+)
            )?
            (?:\.
                (?P<patch>\d+)
            )?
            $
            "#,
        )?;

        let captures = re.captures(s).ok_or(BadNixVersionSpec::InvalidFormat)?;

        let op = captures
            .name("op")
            .ok_or(BadNixVersionSpec::InvalidFormat)?
            .as_str();
        let major: u32 = captures
            .name("major")
            .ok_or(BadNixVersionSpec::InvalidFormat)?
            .as_str()
            .parse()?;
        let minor = captures
            .name("minor")
            .map_or(Ok(0), |m| m.as_str().parse::<u32>())?;
        let patch = captures
            .name("patch")
            .map_or(Ok(0), |m| m.as_str().parse::<u32>())?;

        let nix_version = NixVersion {
            major,
            minor,
            patch,
        };

        match op {
            ">=" => Ok(Gteq(nix_version)),
            "<=" => Ok(Lteq(nix_version)),
            ">" => Ok(Gt(nix_version)),
            "<" => Ok(Lt(nix_version)),
            "!=" => Ok(Neq(nix_version)),
            unknown_op => Err(BadNixVersionSpec::UnknownOperator(unknown_op.to_string())),
        }
    }
}

impl fmt::Display for NixVersionSpec {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NixVersionSpec::Gt(v) => write!(f, ">{}", v),
            NixVersionSpec::Gteq(v) => write!(f, ">={}", v),
            NixVersionSpec::Lt(v) => write!(f, "<{}", v),
            NixVersionSpec::Lteq(v) => write!(f, "<={}", v),
            NixVersionSpec::Neq(v) => write!(f, "!={}", v),
        }
    }
}

impl fmt::Display for NixVersionReq {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            self.specs
                .iter()
                .map(|s| s.to_string())
                .collect::<Vec<_>>()
                .join(", ")
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        assert_eq!(
            NixVersionSpec::from_str(">2.8").unwrap(),
            NixVersionSpec::Gt(NixVersion {
                major: 2,
                minor: 8,
                patch: 0
            })
        );
        assert_eq!(
            NixVersionSpec::from_str(">2").unwrap(),
            NixVersionSpec::Gt(NixVersion {
                major: 2,
                minor: 0,
                patch: 0
            })
        );
    }

    #[test]
    fn test_matches() {
        let req = NixVersionReq::from_str("!=2.9, >2.8").unwrap();

        let version = NixVersion::from_str("2.9.0").unwrap();
        assert!(!req.specs.iter().all(|spec| spec.matches(&version)));
        let version = NixVersion::from_str("2.9.1").unwrap();
        assert!(req.specs.iter().all(|spec| spec.matches(&version)));
    }
}
