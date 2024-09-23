use std::{collections::HashMap, sync::LazyLock};

use nix_rs::flake::url::FlakeUrl;

const BUILTIN_REGISTRY_JSON: &str =
    include_str!(concat!(env!("OM_INIT_REGISTRY"), "/registry.json"));

/// Our builtin registry of templates
pub static BUILTIN_REGISTRY: LazyLock<Registry> =
    LazyLock::new(|| serde_json::from_str(BUILTIN_REGISTRY_JSON).unwrap());

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub struct Registry(pub HashMap<String, FlakeUrl>);
