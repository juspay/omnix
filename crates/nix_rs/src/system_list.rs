//! Dealing with system lists
use std::{collections::HashMap, convert::Infallible, str::FromStr};

use crate::{
    command::{NixCmd, NixCmdError},
    flake::{system::System, url::FlakeUrl},
};
use lazy_static::lazy_static;

lazy_static! {
    /// Builtin list of [SystemsListFlakeRef]
    pub static ref NIX_SYSTEMS: HashMap<String, FlakeUrl> = {
        serde_json::from_str(env!("NIX_SYSTEMS")).unwrap()
    };
}

/// A flake referencing a [SystemsList]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SystemsListFlakeRef(pub FlakeUrl);

impl SystemsListFlakeRef {
    /// Lookup a known [SystemsListFlakeRef] that will not require network calls
    pub fn from_known_system(system: &System) -> Option<Self> {
        NIX_SYSTEMS
            .get(&system.to_string())
            .map(|url| SystemsListFlakeRef(url.clone()))
    }
}

impl FromStr for SystemsListFlakeRef {
    type Err = Infallible;
    fn from_str(s: &str) -> Result<SystemsListFlakeRef, Infallible> {
        let system = System::from(s);
        match SystemsListFlakeRef::from_known_system(&system) {
            Some(url) => Ok(url),
            None => Ok(SystemsListFlakeRef(FlakeUrl(s.to_string()))),
        }
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
        .run_with_args_expecting_json::<T>(&["eval"], &["--impure", "--json", "--expr", &expr])
        .await?;
    Ok(v)
}
