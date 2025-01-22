//! Transform a JSON file with Nix store paths such that the resultant JSON file path will track those paths as dependencies. This requires use of `--impure`.
///
/// Only values of keys called `outPaths` in the JSON will be transformed.
///
/// https://nix.dev/manual/nix/2.23/language/string-context
use super::core::FlakeFn;
use crate::{command::NixCmd, flake::url::FlakeUrl};
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{path::Path, path::PathBuf};

struct AddStringContextFn;

lazy_static! {
    /// URL to our flake function
    static ref FLAKE_ADDSTRINGCONTEXT: FlakeUrl = {
        let path = env!("FLAKE_ADDSTRINGCONTEXT");
        Into::<FlakeUrl>::into(Path::new(path)).with_attr("default")
    };
}

impl FlakeFn for AddStringContextFn {
    type Input = AddStringContextInput;
    type Output = Value; // We don't care to parse the output

    fn flake() -> &'static FlakeUrl {
        &FLAKE_ADDSTRINGCONTEXT
    }
}

/// Input to FlakeMetadata
#[derive(Serialize, Deserialize, Debug)]
struct AddStringContextInput {
    /// The JSON file to process
    jsonfile: FlakeUrl,
}

/// Add string context to `outPath`s in a JSON file.
///
/// Resultant JSON file will track those paths as dependencies. Additionally, an out-link will be created at `out_link` if provided.
pub async fn addstringcontext(
    cmd: &NixCmd,
    jsonfile: &Path,
    out_link: &Path,
) -> Result<PathBuf, super::core::Error> {
    const IMPURE: bool = true; // Our flake.nix uses builtin.storePath

    // We have to use relative paths to avoid a Nix issue on macOS witih /tmp paths.
    let jsonfile_parent = jsonfile.parent().unwrap();
    let jsonfile_name = jsonfile.file_name().unwrap().to_string_lossy();
    let pwd = Some(jsonfile_parent);

    let input = AddStringContextInput {
        jsonfile: FlakeUrl(format!("path:{}", jsonfile_name)),
    };
    let (path_with_string_context, _json_value) =
        AddStringContextFn::call(cmd, false, IMPURE, pwd, Some(out_link), vec![], input).await?;
    Ok(path_with_string_context)
}
