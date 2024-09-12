//! Rust support for invoking <https://github.com/srid/devour-flake>

// TODO: Create a more general version of this module, where a function body is defined in Nix, but FFI invoked (as it were) from Rust.

use anyhow::{bail, Context, Result};
use nix_rs::{command::NixCmd, flake::url::FlakeUrl, store::path::StorePath};
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    process::Stdio,
};
use tokio::io::{AsyncBufReadExt, BufReader};

/// Absolute path to the devour-flake flake source
pub const DEVOUR_FLAKE: &str = env!("DEVOUR_FLAKE");

/// Input arguments to devour-flake
pub struct DevourFlakeInput {
    /// The flake devour-flake will build
    pub flake: FlakeUrl,
    /// The systems it will build for. An empty list means all allowed systems.
    pub systems: Option<FlakeUrl>,
}

/// Output of `devour-flake`
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct DevourFlakeOutput {
    /// The built store paths
    ///
    /// This includes all dependencies if --print-all-dependencies was passed.
    #[serde(rename = "out-paths")]
    pub out_paths: HashSet<StorePath>,

    #[serde(rename = "by-name")]
    pub by_name: HashMap<String, StorePath>,
}

impl DevourFlakeOutput {
    fn from_drv(drv_out: &str) -> anyhow::Result<Self> {
        // Read drv_out file as JSON, decoding it into DevourFlakeOutput
        let out: DevourFlakeOutput = serde_json::from_reader(std::fs::File::open(drv_out)?)
            .context("Failed to parse devour-flake output")?;
        Ok(out)
    }
}

/// Run `devour-flake`
pub async fn devour_flake(
    nixcmd: &NixCmd,
    verbose: bool,
    input: DevourFlakeInput,
    extra_args: Vec<String>,
) -> Result<DevourFlakeOutput> {
    // TODO: Use nix_rs here as well
    // In the context of doing https://github.com/srid/nixci/issues/15
    let devour_flake_url = format!("{}#json", env!("DEVOUR_FLAKE"));
    let mut cmd = nixcmd.command();

    let mut args = vec![
        "build",
        &devour_flake_url,
        "-L",
        "--no-link",
        "--print-out-paths",
        "--override-input",
        "flake",
        &input.flake,
    ];
    // Specify only if the systems is not the default
    if let Some(systems) = input.systems.as_ref() {
        args.extend(&["--override-input", "systems", &systems.0]);
    }
    args.extend(extra_args.iter().map(|s| s.as_str()));
    cmd.args(args);

    nix_rs::command::trace_cmd(&cmd);
    let mut output_fut = cmd.stdout(Stdio::piped()).stderr(Stdio::piped()).spawn()?;
    let stderr_handle = output_fut.stderr.take().unwrap();
    tokio::spawn(async move {
        let mut reader = BufReader::new(stderr_handle).lines();
        while let Some(line) = reader.next_line().await.expect("read stderr") {
            if !verbose {
                if line.starts_with("• Added input") {
                    // Consume the input logging itself
                    reader.next_line().await.expect("read stderr");
                    continue;
                } else if line.starts_with("warning: not writing modified lock file of flake") {
                    continue;
                }
            }
            eprintln!("{}", line);
        }
    });
    let output = output_fut
        .wait_with_output()
        .await
        .context("Unable to spawn devour-flake process")?;
    if output.status.success() {
        let drv_out = String::from_utf8(output.stdout)?;
        let v = DevourFlakeOutput::from_drv(drv_out.trim())?;
        Ok(v)
    } else {
        let exit_code = output.status.code().unwrap_or(1);
        bail!("devour-flake failed to run (exited: {})", exit_code);
    }
}

/// Transform `--override-input` arguments to use `flake/` prefix, which
/// devour_flake expects.
pub fn transform_override_inputs(args: &mut [String]) {
    let mut iter = args.iter_mut().peekable();

    while let Some(arg) = iter.next() {
        if *arg == "--override-input" {
            if let Some(next_arg) = iter.next() {
                *next_arg = format!("flake/{}", next_arg);
            }
        }
    }
}
