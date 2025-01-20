//! Retrieve metadata for a flake.
use super::{functions::FlakeFn, url::FlakeUrl};
use crate::command::NixCmd;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use std::{path::Path, path::PathBuf};

/// Flake metadata computed in Nix.
pub struct FlakeMetadataFn;

lazy_static! {
    /// URL to our flake function
    static ref FLAKE_METADATA: FlakeUrl = {
        let path = env!("FLAKE_METADATA");
        Into::<FlakeUrl>::into(Path::new(path)).with_attr("all")
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
    pub inputs: Vec<FlakeInput>,
}

impl FlakeMetadata {
    /// Get all inputs
    pub fn get_inputs_paths(&self) -> Vec<PathBuf> {
        self.inputs.iter().map(|i| i.path.clone()).collect()
    }
}

/// A flake input
#[derive(Serialize, Deserialize, Debug)]
pub struct FlakeInput {
    /// Unique identifier
    pub name: String,
    /// Local path to the input
    pub path: PathBuf,
}

impl FlakeMetadata {
    /// Get the [FlakeMetadata] for the given flake
    ///
    /// NOTE: This will be `O(n)` where `n` is the count of all flake inputs (transitive). Therefore, this function will be expensive when used in large flakes.
    pub async fn recursive_evaluate(
        cmd: &NixCmd,
        flake_url: &FlakeUrl,
    ) -> Result<(PathBuf, FlakeMetadata), crate::flake::functions::Error> {
        let (store_path, v) = FlakeMetadataFn::call(
            cmd,
            false,
            vec![],
            FlakeMetadataInput {
                flake: flake_url.clone(),
            },
        )
        .await?;
        Ok((store_path, v))
    }
}
