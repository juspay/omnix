/// Store path management
use std::{convert::Infallible, fmt, path::PathBuf, str::FromStr};

use serde_with::{DeserializeFromStr, SerializeDisplay};

/// Represents a path in the Nix store, see: <https://zero-to-nix.com/concepts/nix-store#store-paths>
#[derive(
    Debug, Ord, PartialOrd, Eq, PartialEq, Clone, Hash, DeserializeFromStr, SerializeDisplay,
)]
pub enum StorePath {
    /// Derivation path (ends with `.drv`).
    Drv(PathBuf),
    /// Other paths in the Nix store, such as build outputs.
    /// This won't be a derivation path.
    Other(PathBuf),
}

impl FromStr for StorePath {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(StorePath::new(PathBuf::from(s)))
    }
}

impl From<&StorePath> for PathBuf {
    fn from(sp: &StorePath) -> Self {
        sp.as_path().clone()
    }
}

impl StorePath {
    pub fn new(path: PathBuf) -> Self {
        if path.ends_with(".drv") {
            StorePath::Drv(path)
        } else {
            StorePath::Other(path)
        }
    }

    pub fn as_path(&self) -> &PathBuf {
        match self {
            StorePath::Drv(p) => p,
            StorePath::Other(p) => p,
        }
    }
}

impl fmt::Display for StorePath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_path().display())
    }
}
