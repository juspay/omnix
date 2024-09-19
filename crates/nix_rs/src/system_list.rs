//! Dealing with system lists
use std::{collections::HashMap, str::FromStr};

use crate::{
    command::{NixCmd, NixCmdError},
    flake::{system::System, url::FlakeUrl},
};
use lazy_static::lazy_static;

/// A flake referencing a [SystemsList]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SystemsListFlakeRef(pub FlakeUrl);

lazy_static! {
    /// As a HashMap<String, String>
    pub static ref NIX_SYSTEMS: HashMap<String, FlakeUrl> = {
        serde_json::from_str(env!("NIX_SYSTEMS")).unwrap()
    };
}

impl FromStr for SystemsListFlakeRef {
    type Err = String;
    fn from_str(s: &str) -> Result<SystemsListFlakeRef, String> {
        // Systems lists recognized by `github:nix-system/*`
        let url = if let Some(nix_system_flake) = NIX_SYSTEMS.get(s) {
            nix_system_flake.clone()
        } else {
            FlakeUrl(s.to_string())
        };
        Ok(SystemsListFlakeRef(url))
    }
}

/// A list of [System]s
pub struct SystemsList(pub Vec<System>);

impl SystemsList {
    /// Load the list of systems from a [SystemsListFlakeRef]
    pub async fn from_flake(cmd: &NixCmd, url: &SystemsListFlakeRef) -> Result<Self, NixCmdError> {
        // Nix eval, and then return the systems
        match SystemsList::from_known_flake(url) {
            Some(systems) => Ok(systems),
            None => SystemsList::from_remote_flake(cmd, url).await,
        }
    }

    async fn from_remote_flake(
        cmd: &NixCmd,
        url: &SystemsListFlakeRef,
    ) -> Result<Self, NixCmdError> {
        let systems = nix_import_flake::<Vec<System>>(cmd, &url.0).await?;
        Ok(SystemsList(systems))
    }

    /// Handle known repos of <https://github.com/nix-systems> thereby avoiding
    /// network calls.
    fn from_known_flake(url: &SystemsListFlakeRef) -> Option<Self> {
        let system = NIX_SYSTEMS
            .iter()
            .find_map(|(v, u)| if u == &url.0 { Some(v) } else { None })?;
        Some(SystemsList(vec![system.clone().into()]))
    }
}

/// Evaluate `import <flake-url>` and return the result JSON parsed.
async fn nix_import_flake<T>(cmd: &NixCmd, url: &FlakeUrl) -> Result<T, NixCmdError>
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
}
