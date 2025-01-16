//! Retrieve metadata for a flake.
use super::{functions::FlakeFn, url::FlakeUrl};
use crate::command::NixCmd;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::PathBuf, path::Path};

/// Flake metadata
pub struct FlakeMetadataFn;

lazy_static! {
    /// devour flake URL
    static ref FLAKE_METADATA: FlakeUrl = {
        let path = env!("FLAKE_METADATA");
        Into::<FlakeUrl>::into(Path::new(path)).with_attr("default")
    };
}

impl FlakeFn for FlakeMetadataFn {
    type Input = FlakeMetadataInput;
    type Output = FlakeMetadata;

    fn flake() -> &'static FlakeUrl {
        &FLAKE_METADATA
    }
}

/// Input to FlakeMetadata
#[derive(Serialize, Deserialize, Debug)]
pub struct FlakeMetadataInput {
    /// The flake to operate on
    pub flake: FlakeUrl,
}

/// Flake metadata
///
/// See [Nix doc](https://nix.dev/manual/nix/2.18/command-ref/new-cli/nix3-flake-metadata)
#[derive(Serialize, Deserialize, Debug)]
pub struct FlakeMetadata {
    /// Store path to this flake
    pub flake: PathBuf,

    /// Store path to each flake input
    pub inputs: HashMap<String, PathBuf>,
}

impl FlakeMetadata {
    /// Runs `nix flake metadata --json` for a given flake url
    pub async fn from_nix(
        cmd: &NixCmd,
        flake_url: &FlakeUrl,
    ) -> Result<FlakeMetadata, crate::flake::functions::Error> {
        let v = FlakeMetadataFn::call(
            cmd,
            false,
            vec![],
            FlakeMetadataInput {
                flake: flake_url.clone(),
            },
        )
        .await?;
        Ok(v)
    }
}
