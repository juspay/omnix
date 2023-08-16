use std::{
    fmt::{Display, Formatter},
    str::FromStr,
};

use leptos::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FlakeUrl(String);

impl From<&str> for FlakeUrl {
    fn from(url: &str) -> Self {
        url.to_string().into()
    }
}

impl From<String> for FlakeUrl {
    fn from(url: String) -> Self {
        Self(url)
    }
}

impl FromStr for FlakeUrl {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();
        if s.is_empty() {
            Err("Empty string is not a valid Flake URL".to_string())
        } else {
            Ok(s.into())
        }
    }
}

impl IntoView for FlakeUrl {
    fn into_view(self, cx: Scope) -> View {
        self.0.into_view(cx)
    }
}

impl Display for FlakeUrl {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
