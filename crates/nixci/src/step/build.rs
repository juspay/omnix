use std::collections::HashSet;

use crate::command::build::BuildCommand;
use crate::nix::devour_flake::DevourFlakeOutput;
use colored::Colorize;
use nix_rs::{
    command::NixCmd,
    config::NixConfig,
    flake::url::FlakeUrl,
    store::{DrvOut, NixStoreCmd, StorePath},
};
use tracing::instrument;

use crate::{config, nix};

pub async fn nixci_build(
    cmd: &NixCmd,
    verbose: bool,
    build_cmd: &BuildCommand,
    cfg: &config::Config,
    nix_config: &NixConfig,
) -> anyhow::Result<Vec<StorePath>> {
    let mut all_outs = HashSet::new();

    let all_devour_flake_outs = nixci_subflakes(cmd, verbose, build_cmd, cfg, nix_config).await?;

    if build_cmd.print_all_dependencies {
        let all_deps = NixStoreCmd
            .fetch_all_deps(all_devour_flake_outs.into_iter().collect())
            .await?;
        all_outs.extend(all_deps.into_iter());
    } else {
        let store_paths: HashSet<StorePath> = all_devour_flake_outs
            .into_iter()
            .map(DrvOut::as_store_path)
            .collect();
        all_outs.extend(store_paths);
    }

    for out in &all_outs {
        println!("{}", out);
    }

    Ok(all_outs.into_iter().collect())
}

async fn nixci_subflakes(
    cmd: &NixCmd,
    verbose: bool,
    build_cmd: &BuildCommand,
    cfg: &config::Config,
    nix_config: &NixConfig,
) -> anyhow::Result<HashSet<DrvOut>> {
    let mut result = HashSet::new();
    let systems = build_cmd.get_systems(cmd, nix_config).await?;

    for (subflake_name, subflake) in &cfg.subflakes.0 {
        let name = format!("{}.{}", cfg.ref_.selected_name, subflake_name).italic();
        if subflake.skip {
            tracing::info!("🍊 {} {}", name, "skipped (deselected out)".dimmed());
            continue;
        }
        tracing::info!("🍎 {}", name);
        if subflake.can_build_on(&systems) {
            let outs = nixci_subflake(
                cmd,
                verbose,
                build_cmd,
                &cfg.ref_.flake_url,
                subflake_name,
                subflake,
            )
            .await?;
            result.extend(outs.0);
        } else {
            tracing::info!(
                "🍊 {} {}",
                name,
                "skipped (cannot build on this system)".dimmed()
            );
        }
    }

    Ok(result)
}

#[instrument(skip(build_cmd, url))]
async fn nixci_subflake(
    cmd: &NixCmd,
    verbose: bool,
    build_cmd: &BuildCommand,
    url: &FlakeUrl,
    subflake_name: &str,
    subflake: &config::SubflakeConfig,
) -> anyhow::Result<DevourFlakeOutput> {
    if subflake.override_inputs.is_empty() {
        nix::lock::nix_flake_lock_check(cmd, &url.sub_flake_url(subflake.dir.clone())).await?;
    }

    let nix_args = subflake.nix_build_args_for_flake(build_cmd, url);
    let outs = nix::devour_flake::devour_flake(cmd, verbose, nix_args).await?;
    Ok(outs)
}
