use leptos::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FlakeUrl(String);

impl FlakeUrl {
    pub fn new(url: impl Into<String>) -> Self {
        Self(url.into())
    }
}

impl ToString for FlakeUrl {
    fn to_string(&self) -> String {
        self.0.clone()
    }
}

impl IntoView for FlakeUrl {
    fn into_view(self, cx: Scope) -> View {
        self.0.into_view(cx)
    }
}
