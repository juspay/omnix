//! Rust support for invoking <https://github.com/srid/devour-flake>

use lazy_static::lazy_static;
use nix_rs::{
    flake::{functions::core::FlakeFn, url::FlakeUrl},
    store::path::StorePath,
};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::Path};

/// Devour all outputs of a flake producing their store paths
pub struct DevourFlake;

lazy_static! {
    /// devour flake URL
    static ref DEVOUR_FLAKE: FlakeUrl = {
        let path = env!("DEVOUR_FLAKE");
        Into::<FlakeUrl>::into(Path::new(path)).with_attr("json")
    };
}

impl FlakeFn for DevourFlake {
    type Input = DevourFlakeInput;
    type Output = DevourFlakeOutput;

    fn flake() -> &'static FlakeUrl {
        &DEVOUR_FLAKE
    }

    fn init(out: &mut DevourFlakeOutput) {
        // Remove duplicates, which is possible in user's flake
        // e.g., when doing `packages.foo = self'.packages.default`
        out.out_paths.sort();
        out.out_paths.dedup();
    }
}

/// Input arguments to devour-flake
#[derive(Serialize)]
pub struct DevourFlakeInput {
    /// The flake whose outputs will be built
    pub flake: FlakeUrl,
    /// The systems it will build for. An empty list means all allowed systems.
    pub systems: Option<FlakeUrl>,
}

/// Output of `devour-flake`
#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DevourFlakeOutput {
    /// The built store paths
    #[serde(rename = "outPaths")]
    pub out_paths: Vec<StorePath>,

    /// Output paths indexed by name (or pname) of the path if any
    #[serde(rename = "byName")]
    pub by_name: HashMap<String, StorePath>,
}
