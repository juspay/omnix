//! Rust module defining the spec for [NixVersion]

use std::{fmt, str::FromStr};

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::version::{BadNixVersion, NixVersion};

/// Specifies a version requirement for Nix
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

/// A collection of version requirements
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NixVersionReq {
    /// List of version specifications
    pub specs: Vec<NixVersionSpec>,
}

/// Errors that can occur while parsing or validating version specifications
#[derive(Error, Debug)]
pub enum BadNixVersionSpec {
    /// Error occurred while parsing the Nix version
    #[error("Nix version error while parsing spec: {0}")]
    NixVersion(#[from] BadNixVersion),

    /// The version requirements cannot be satisfied together
    #[error("Version bounds are not satisfiable")]
    UnsatisfiableBounds,

    /// An unknown comparison operator was used
    #[error("Unknown operator in the Nix version spec: {0}")]
    UnknownOperator(String),
}

impl NixVersionReq {
    /// Parses a comma-separated string of version requirements
    ///
    /// # Example
    /// ```
    /// use nix_rs::version_spec::NixVersionReq;
    /// let req = NixVersionReq::parse(">2.8, <3.0, !=2.9").unwrap();
    /// ```
    pub fn parse(s: &str) -> Result<Self, BadNixVersionSpec> {
        let specs = s
            .split(',')
            .map(str::trim)
            .map(NixVersionSpec::from_str)
            .collect::<Result<Vec<_>, _>>()?;

        Self::check(&specs)?;
        Ok(NixVersionReq { specs })
    }

    /// Checks if all version specifications are satisfiable
    ///
    /// Returns Err if the requirements are logically impossible to satisfy
    fn check(specs: &[NixVersionSpec]) -> Result<(), BadNixVersionSpec> {
        let max_lower = specs
            .iter()
            .filter_map(|spec| match spec {
                NixVersionSpec::Gt(v) => Some(v),
                _ => None,
            })
            .max();

        let min_upper = specs
            .iter()
            .filter_map(|spec| match spec {
                NixVersionSpec::Lt(v) => Some(v),
                _ => None,
            })
            .min();

        // Check basic bounds
        if let (Some(lower), Some(upper)) = (max_lower, min_upper) {
            if lower >= upper {
                return Err(BadNixVersionSpec::UnsatisfiableBounds);
            }
        }

        // Check if all the versions part of neq spec fall within the bounds
        for spec in specs {
            if let NixVersionSpec::Neq(v) = spec {
                if let Some(lower) = max_lower {
                    if v <= lower {
                        return Err(BadNixVersionSpec::UnsatisfiableBounds);
                    }
                }
                if let Some(upper) = min_upper {
                    if v >= upper {
                        return Err(BadNixVersionSpec::UnsatisfiableBounds);
                    }
                }
            }
        }

        Ok(())
    }
}

impl NixVersionSpec {
    /// Checks if a given Nix version satisfies this version specification
    pub fn matches(&self, version: &NixVersion) -> bool {
        match self {
            NixVersionSpec::Gt(v) => version > v,
            NixVersionSpec::Gteq(v) => version >= v,
            NixVersionSpec::Lt(v) => version < v,
            NixVersionSpec::Lteq(v) => version < v,
            NixVersionSpec::Neq(v) => version != v,
        }
    }
}

impl FromStr for NixVersionSpec {
    type Err = BadNixVersionSpec;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use NixVersionSpec::{Gt, Gteq, Lt, Lteq, Neq};

        let s = s.trim();
        match s.get(..2) {
            Some("!=") => Ok(Neq(NixVersion::from_str(&s[2..])?)),
            Some(">=") => Ok(Gteq(NixVersion::from_str(&s[2..])?)),
            Some("<=") => Ok(Lteq(NixVersion::from_str(&s[2..])?)),
            Some(s2) if s2.starts_with('>') => Ok(Gt(NixVersion::from_str(&s[1..])?)),
            Some(s2) if s2.starts_with('<') => Ok(Lt(NixVersion::from_str(&s[1..])?)),
            _ => Err(BadNixVersionSpec::UnknownOperator(s.to_string())),
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
        let mut specs = self.specs.iter();

        if let Some(first) = specs.next() {
            write!(f, "{}", first)?;
            for spec in specs {
                write!(f, ", {}", spec)?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unsatisfiable_bounds() {
        assert!(matches!(
            NixVersionReq::parse(">2.8, <2.6"),
            Err(BadNixVersionSpec::UnsatisfiableBounds)
        ));

        assert!(matches!(
            NixVersionReq::parse(">2.8, <2.23, !=2.24"),
            Err(BadNixVersionSpec::UnsatisfiableBounds)
        ));
    }

    #[test]
    fn test_matches() {
        let req = NixVersionReq::parse(">2.8").unwrap();

        let version = NixVersion::from_str("2.9.0").unwrap();
        assert!(req.specs.iter().all(|spec| spec.matches(&version)));
        let version = NixVersion::from_str("2.7.0").unwrap();
        assert!(!req.specs.iter().all(|spec| spec.matches(&version)));

        let req = NixVersionReq::parse("!=2.9, >2.8").unwrap();

        let version = NixVersion::from_str("2.9.0").unwrap();
        assert!(!req.specs.iter().all(|spec| spec.matches(&version)));
        let version = NixVersion::from_str("2.9.1").unwrap();
        assert!(req.specs.iter().all(|spec| spec.matches(&version)));
    }
}
