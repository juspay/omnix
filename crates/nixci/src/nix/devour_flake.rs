//! Rust support for invoking <https://github.com/srid/devour-flake>

// TODO: Create a more general version of this module, where a function body is defined in Nix, but FFI invoked (as it were) from Rust.

use anyhow::{bail, Context, Result};
use nix_rs::{command::NixCmd, flake::url::FlakeUrl, store::StorePath};
use std::{collections::HashSet, path::PathBuf, process::Stdio};
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
pub struct DevourFlakeOutput(pub HashSet<StorePath>);

impl DevourFlakeOutput {
    fn from_drv(drv_out: &str) -> anyhow::Result<Self> {
        let raw_output = std::fs::read_to_string(drv_out)?;
        let outs = raw_output.split_ascii_whitespace();
        let outs: HashSet<StorePath> = outs.map(|s| StorePath::new(PathBuf::from(s))).collect();
        if outs.is_empty() {
            bail!(
                "devour-flake produced an outpath ({}) with no outputs",
                drv_out
            );
        } else {
            Ok(DevourFlakeOutput(outs))
        }
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
    let devour_flake_url = format!("{}#default", env!("DEVOUR_FLAKE"));
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
