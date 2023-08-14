//! Rust module for `nix --version`
use leptos::*;
use serde::{Deserialize, Serialize};
use std::fmt;
#[cfg(feature = "ssr")]
use tracing::instrument;

/// Nix version as parsed from `nix --version`
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct NixVersion {
    major: u32,
    minor: u32,
    patch: u32,
}

/// Parse the string output of `nix --version` into a [NixVersion]
pub fn parse_nix_version(version_str: &str) -> Option<NixVersion> {
    let components: Vec<&str> = version_str.split_whitespace().collect();

    if components.len() >= 2 {
        let version = components[components.len() - 1];
        let version_parts: Vec<&str> = version.split('.').collect();

        if version_parts.len() == 3 {
            let major = version_parts[0].parse().ok()?;
            let minor = version_parts[1].parse().ok()?;
            let patch = version_parts[2].parse().ok()?;

            Some(NixVersion {
                major,
                minor,
                patch,
            })
        } else {
            None
        }
    } else {
        None
    }
}

/// Get the output of `nix --version`
#[cfg(feature = "ssr")]
#[instrument(name = "version")]
pub async fn run_nix_version() -> Result<NixVersion, ServerFnError> {
    use tokio::process::Command;
    let mut cmd = Command::new("nix");
    cmd.arg("--version");
    let stdout: Vec<u8> = crate::command::run_command(&mut cmd).await?;
    let v = parse_nix_version(std::str::from_utf8(&stdout).unwrap()).unwrap();
    Ok(v)
}

/// The HTML view for [NixVersion]
impl IntoView for NixVersion {
    fn into_view(self, cx: Scope) -> View {
        view! { cx,
            <div class="p-1 my-1 rounded bg-primary-50">
                <pre>{self.major} . {self.minor} . {self.patch}</pre>
            </div>
        }
        .into_view(cx)
    }
}

/// The String view for [NixVersion]
impl fmt::Display for NixVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

#[cfg(feature = "ssr")]
#[tokio::test]
async fn test_run_nix_version() {
    let nix_version = run_nix_version().await.unwrap();
    println!("Nix version: {}", nix_version);
}
