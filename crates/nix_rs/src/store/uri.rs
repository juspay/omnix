/// Store URI management
use std::{fmt, str::FromStr};

use serde::{Deserialize, Serialize};
use thiserror::Error;

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

#[derive(Error, Debug)]
pub enum StoreURIParseError {
    #[error("Invalid URI format")]
    InvalidFormat,
    #[error("Unsupported scheme: {0}")]
    UnsupportedScheme(String),
    #[error("Missing host")]
    MissingHost,
}

impl StoreURI {
    pub fn parse(uri: &str) -> Result<Self, StoreURIParseError> {
        let (scheme, rest) = uri
            .split_once("://")
            .ok_or(StoreURIParseError::InvalidFormat)?;

        match scheme {
            "ssh" => {
                let (user, host) = rest
                    .split_once('@')
                    .map(|(u, h)| (Some(u.to_string()), h))
                    .unwrap_or((None, rest));

                if host.is_empty() {
                    return Err(StoreURIParseError::MissingHost);
                }

                Ok(StoreURI::SSH(SSHStoreURI {
                    user,
                    host: host.to_string(),
                }))
            }
            // Add future schemes here
            _ => Err(StoreURIParseError::UnsupportedScheme(scheme.to_string())),
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
