//! Store URI management
use std::{fmt, str::FromStr};

use serde::{Deserialize, Serialize};
use thiserror::Error;
use url::Url;

/// Nix Store URI
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum StoreURI {
    /// Remote SSH store
    SSH(SSHStoreURI),
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
                let store_uri = SSHStoreURI { user, host };
                Ok(StoreURI::SSH(store_uri))
            }
            // Add future schemes here
            scheme => Err(StoreURIParseError::UnsupportedScheme(scheme.to_string())),
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
            StoreURI::SSH(uri) => {
                write!(f, "ssh://{}", uri)
            }
        }
    }
}
