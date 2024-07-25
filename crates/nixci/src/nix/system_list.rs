use std::str::FromStr;

use anyhow::Result;
use nix_rs::{
    command::{NixCmd, NixCmdError},
    flake::{system::System, url::FlakeUrl},
};

/// A flake URL that references a list of systems ([SystemsList])
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SystemsListFlakeRef(pub FlakeUrl);

impl FromStr for SystemsListFlakeRef {
    type Err = String;
    fn from_str(s: &str) -> std::result::Result<SystemsListFlakeRef, String> {
        // Systems lists recognized by `github:nix-system/*`
        let known_nix_systems = [
            "aarch64-darwin",
            "aarch64-linux",
            "x86_64-darwin",
            "x86_64-linux",
        ];
        let url = if known_nix_systems.contains(&s) {
            format!("github:nix-systems/{}", s)
        } else {
            s.to_string()
        };
        Ok(SystemsListFlakeRef(FlakeUrl(url)))
    }
}

pub struct SystemsList(pub Vec<System>);

impl SystemsList {
    pub async fn from_flake(cmd: &NixCmd, url: &SystemsListFlakeRef) -> Result<Self> {
        // Nix eval, and then return the systems
        match SystemsList::from_known_flake(url) {
            Some(systems) => Ok(systems),
            None => SystemsList::from_remote_flake(cmd, url).await,
        }
    }

    async fn from_remote_flake(cmd: &NixCmd, url: &SystemsListFlakeRef) -> Result<Self> {
        let systems = nix_import_flake::<Vec<System>>(cmd, &url.0).await?;
        Ok(SystemsList(systems))
    }

    /// Handle known repos of https://github.com/nix-systems thereby avoiding
    /// network calls.
    fn from_known_flake(url: &SystemsListFlakeRef) -> Option<Self> {
        match url.0 .0.as_str() {
            "github:nix-systems/empty" => Some(SystemsList(vec![])),
            "github:nix-systems/default-darwin" => Some(SystemsList(vec![
                "aarch64-darwin".into(),
                "x86_64-darwin".into(),
            ])),
            "github:nix-systems/default-linux" => Some(SystemsList(vec![
                "aarch64-linux".into(),
                "x86_64-linux".into(),
            ])),
            "github:nix-systems/aarch64-darwin" => Some(SystemsList(vec!["aarch64-darwin".into()])),
            "github:nix-systems/aarch64-linux" => Some(SystemsList(vec!["aarch64-linux".into()])),
            "github:nix-systems/x86_64-darwin" => Some(SystemsList(vec!["x86_64-darwin".into()])),
            "github:nix-systems/x86_64-linux" => Some(SystemsList(vec!["x86_64-linux".into()])),
            _ => None,
        }
    }
}

pub async fn nix_import_flake<T>(cmd: &NixCmd, url: &FlakeUrl) -> Result<T, NixCmdError>
where
    T: Default + serde::de::DeserializeOwned,
{
    let flake_path =
        nix_eval_impure_expr::<String>(cmd, format!("builtins.getFlake \"{}\"", url.0)).await?;
    let v = nix_eval_impure_expr(cmd, format!("import {}", flake_path)).await?;
    Ok(v)
}

async fn nix_eval_impure_expr<T>(cmd: &NixCmd, expr: String) -> Result<T, NixCmdError>
where
    T: Default + serde::de::DeserializeOwned,
{
    let v = cmd
        .run_with_args_expecting_json::<T>(&["eval", "--impure", "--json", "--expr", &expr])
        .await?;
    Ok(v)
}

#[cfg(test)]
#[cfg(feature = "integration_test")]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_empty_systems_list() {
        let systems = SystemsList::from_flake(
            &NixCmd::default(),
            &SystemsListFlakeRef(FlakeUrl("github:nix-systems/empty".to_string())),
        )
        .await
        .unwrap();
        assert_eq!(systems.0, vec![]);
    }

    #[tokio::test]
    async fn test_systems_list() {
        assert_systems_list(
            "github:nix-systems/default-linux",
            vec!["aarch64-linux".into(), "x86_64-linux".into()],
        )
        .await;
        assert_systems_list(
            "github:nix-systems/default-darwin",
            vec!["aarch64-darwin".into(), "x86_64-darwin".into()],
        )
        .await;
        assert_systems_list(
            "github:nix-systems/aarch64-linux",
            vec!["aarch64-linux".into()],
        )
        .await;
        assert_systems_list(
            "github:nix-systems/aarch64-darwin",
            vec!["aarch64-darwin".into()],
        )
        .await;
        assert_systems_list(
            "github:nix-systems/x86_64-linux",
            vec!["x86_64-linux".into()],
        )
        .await;
        assert_systems_list(
            "github:nix-systems/x86_64-darwin",
            vec!["x86_64-darwin".into()],
        )
        .await;
        assert_systems_list("github:nix-systems/empty", vec![]).await;
    }

    async fn assert_systems_list(url: &str, expected: Vec<System>) {
        let cmd = NixCmd::default();
        let systems = SystemsList::from_flake(&cmd, &SystemsListFlakeRef(url.into()))
            .await
            .unwrap();
        assert_eq!(systems.0, expected);
    }
}
