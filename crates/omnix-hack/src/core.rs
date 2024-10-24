use anyhow::Context;
use std::path::Path;

use nix_rs::{flake::url::FlakeUrl, info::NixInfo};
use omnix_common::markdown::print_markdown;
use omnix_health::{traits::Checkable, NixHealth};

use crate::config::HackConfig;

pub async fn hack_on(dir: &Path) -> anyhow::Result<()> {
    let dir = dir.canonicalize()?;
    let here_flake: FlakeUrl = Into::<FlakeUrl>::into(dir.as_ref());
    let cfg = HackConfig::from_flake(&here_flake).await?;

    // TODO: cachix check

    // Run relevant `om health` checks
    let health = NixHealth::from_flake(&here_flake).await?;
    let nix_info = NixInfo::get()
        .await
        .as_ref()
        .with_context(|| "Unable to gather nix info")?;
    let relevant_checks: Vec<&'_ dyn Checkable> = vec![
        &health.nix_version,
        &health.rosetta,
        &health.max_jobs,
        // TODO: Run this only when a cache is configured
        &health.trusted_users,
        &health.caches,
    ];
    for check_kind in relevant_checks.into_iter() {
        for check in check_kind.check(nix_info, Some(&here_flake)) {
            if !check.result.green() {
                check.tracing_log().await?;
                if !check.result.green() && check.required {
                    tracing::error!("ERROR: Your Nix invironment is not properly setup. Run `om health` for details.");
                    anyhow::bail!("Cannot proceed");
                };
            };
        }
    }
    if !health.caches.required.is_empty() {
        // TODO: Auto-resolve some problems; like running 'cachix use' automatically
    };
    tracing::info!("Healthy");

    eprintln!();
    print_markdown(&dir, &cfg.readme.get_markdown()).await?;

    Ok(())
}
