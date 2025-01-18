//! Store URI management
use std::{fmt, str::FromStr};

use serde::{Deserialize, Serialize};
use thiserror::Error;
use url::Url;

/// Refers to a Nix store somewhere.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum StoreURI {
    /// Nix store accessible over SSH.
    SSH(SSHStoreURI, Opts),
}

/// User passed options for a store URI
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Opts {
    /// Whether to copy all flake inputs recursively
    ///
    /// If disabled, we copy only the flake source itself. Enabling this option is useful when there are private Git inputs but the target machine does not have access to them.
    #[serde(rename = "copy-inputs", default = "bool::default")]
    pub copy_inputs: bool,

    /// Whether to copy built outputs back to local store
    #[serde(rename = "copy-outputs", default = "bool_true")]
    pub copy_outputs: bool,
}

fn bool_true() -> bool {
    true
}

/// Remote SSH store URI
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SSHStoreURI {
    /// SSH user
    pub user: Option<String>,
    /// SSH host
    pub host: String,
}

/// Error parsing a store URI
#[derive(Error, Debug)]
pub enum StoreURIParseError {
    /// Parse error
    #[error(transparent)]
    ParseError(#[from] url::ParseError),
    /// Unsupported scheme
    #[error("Unsupported scheme: {0}")]
    UnsupportedScheme(String),
    /// Missing host
    #[error("Missing host")]
    MissingHost,
    /// Query string parse error
    #[error(transparent)]
    QueryParseError(#[from] serde_qs::Error),
}

impl StoreURI {
    /// Parse a Nix store URI
    ///
    /// Currently only supports `ssh` scheme
    pub fn parse(uri: &str) -> Result<Self, StoreURIParseError> {
        let url = Url::parse(uri)?;
        match url.scheme() {
            "ssh" => {
                let host = url
                    .host_str()
                    .ok_or(StoreURIParseError::MissingHost)?
                    .to_string();
                let user = if !url.username().is_empty() {
                    Some(url.username().to_string())
                } else {
                    None
                };
                let opts = serde_qs::from_str(url.query().unwrap_or(""))?;
                let ssh_uri = SSHStoreURI { user, host };
                let store_uri = StoreURI::SSH(ssh_uri, opts);
                Ok(store_uri)
            }
            // Add future schemes here
            scheme => Err(StoreURIParseError::UnsupportedScheme(scheme.to_string())),
        }
    }

    /// Get the options for this store URI
    pub fn get_options(&self) -> &Opts {
        match self {
            StoreURI::SSH(_, opts) => opts,
        }
    }
}

impl FromStr for StoreURI {
    type Err = StoreURIParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        StoreURI::parse(s)
    }
}

impl fmt::Display for SSHStoreURI {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(user) = &self.user {
            write!(f, "{}@{}", user, self.host)
        } else {
            write!(f, "{}", self.host)
        }
    }
}
impl fmt::Display for StoreURI {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StoreURI::SSH(uri, _opts) => {
                // This should construct a valid store URI.
                write!(f, "ssh://{}", uri)
            }
        }
    }
}
