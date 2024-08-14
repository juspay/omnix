use std::collections::HashSet;

use colored::Colorize;
use nix_rs::{
    command::NixCmd,
    config::NixConfig,
    flake::url::FlakeUrl,
    store::{NixStoreCmd, StorePath},
};
use tracing::instrument;

use crate::{
    command::build::BuildCommand,
    config::{core::Config, subflake::SubflakeConfig},
    nix,
    nix::devour_flake::DevourFlakeOutput,
};

pub async fn build_flake(
    cmd: &NixCmd,
    verbose: bool,
    build_cmd: &BuildCommand,
    cfg: &Config,
    nix_config: &NixConfig,
) -> anyhow::Result<Vec<StorePath>> {
    // Build all subflakes using devour-flake
    let devour_flake_outs = build_subflakes(cmd, verbose, build_cmd, cfg, nix_config)
        .await?
        .into_iter()
        .collect();

    if build_cmd.print_all_dependencies {
        // Handle --print-all-dependencies
        Ok(NixStoreCmd.fetch_all_deps(devour_flake_outs).await?)
    } else {
        Ok(devour_flake_outs)
    }
}

async fn build_subflakes(
    cmd: &NixCmd,
    verbose: bool,
    build_cmd: &BuildCommand,
    cfg: &Config,
    nix_config: &NixConfig,
) -> anyhow::Result<HashSet<StorePath>> {
    let mut result = HashSet::new();
    let systems = build_cmd.get_systems(cmd, nix_config).await?;

    for (subflake_name, subflake) in &cfg.subflakes.0 {
        let name = format!("{}.{}", cfg.ref_.selected_name, subflake_name).italic();

        if subflake.skip {
            tracing::info!("🍊 {} {}", name, "skipped (deselected out)".dimmed());
            continue;
        }

        let compatible_system = subflake.can_build_on(&systems);
        if !compatible_system {
            tracing::info!(
                "🍊 {} {}",
                name,
                "skipped (cannot build on this system)".dimmed()
            );
            continue;
        }

        tracing::info!("🍎 {}", name);
        let outs = build_subflake(
            cmd,
            verbose,
            build_cmd,
            &cfg.ref_.flake_url,
            subflake_name,
            subflake,
        )
        .await?;
        result.extend(outs.0);
    }

    Ok(result)
}

#[instrument(skip(build_cmd, url))]
async fn build_subflake(
    cmd: &NixCmd,
    verbose: bool,
    build_cmd: &BuildCommand,
    url: &FlakeUrl,
    subflake_name: &str,
    subflake: &SubflakeConfig,
) -> anyhow::Result<DevourFlakeOutput> {
    // Check that `flake.lock` is not out of date.
    if subflake.override_inputs.is_empty() {
        let sub_flake_url = url.sub_flake_url(subflake.dir.clone());
        nix::lock::nix_flake_lock_check(cmd, &sub_flake_url).await?;
    }

    // Run devour-flake to do the actual build.
    let nix_args = nix_build_args_for_subflake(subflake, build_cmd, url);
    nix::devour_flake::devour_flake(cmd, verbose, nix_args).await
}

/// Return the devour-flake `nix build` arguments for building all the outputs in this
/// subflake configuration.
fn nix_build_args_for_subflake(
    subflake: &SubflakeConfig,
    build_cmd: &BuildCommand,
    flake_url: &FlakeUrl,
) -> Vec<String> {
    let mut args = vec![flake_url.sub_flake_url(subflake.dir.clone()).0];

    for (k, v) in &subflake.override_inputs {
        args.extend_from_slice(&[
            "--override-input".to_string(),
            format!("flake/{}", k),
            v.0.to_string(),
        ])
    }

    // devour-flake already uses this, so no need to override.
    if build_cmd.systems.0 .0 != "github-nix-systems/empty" {
        args.extend_from_slice(&[
            "--override-input".to_string(),
            "systems".to_string(),
            build_cmd.systems.0 .0.clone(),
        ])
    }

    args.extend(build_cmd.extra_nix_build_args.iter().cloned());

    args
}
