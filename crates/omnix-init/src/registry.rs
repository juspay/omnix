use lazy_static::lazy_static;
use std::{collections::HashMap, path::Path};

use nix_rs::{
    command::{NixCmd, NixCmdError},
    flake::{command::FlakeOptions, eval::nix_eval, url::FlakeUrl},
};
use tokio::sync::OnceCell;

lazy_static! {
    /// The registry flake
    pub static ref OM_INIT_REGISTRY: FlakeUrl = {
        let path = env!("OM_INIT_REGISTRY");
        Into::<FlakeUrl>::into(Path::new(path)).with_attr("registry")
    };
}

/// Our builtin registry of templates
static BUILTIN_REGISTRY: OnceCell<Result<Registry, NixCmdError>> = OnceCell::const_new();

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub struct Registry(pub HashMap<String, FlakeUrl>);

pub async fn get(nixcmd: &NixCmd) -> &'static Result<Registry, NixCmdError> {
    BUILTIN_REGISTRY
        .get_or_init(|| async {
            let registry =
                nix_eval::<Registry>(nixcmd, &FlakeOptions::default(), &OM_INIT_REGISTRY).await?;
            Ok(registry)
        })
        .await
}
