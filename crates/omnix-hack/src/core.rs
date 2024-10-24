use anyhow::Context;
use std::path::{Path, PathBuf};

use nix_rs::{flake::url::FlakeUrl, info::NixInfo};
use omnix_common::markdown::print_markdown;
use omnix_health::{traits::Checkable, NixHealth};

use crate::config::HackConfig;

pub struct Project {
    pub dir: PathBuf,
    pub flake: FlakeUrl,
    pub cfg: HackConfig,
}

impl Project {
    pub async fn new(dir: &Path) -> anyhow::Result<Self> {
        let dir = dir.canonicalize()?;
        let flake: FlakeUrl = Into::<FlakeUrl>::into(dir.as_ref());
        let cfg = HackConfig::from_flake(&flake).await?;
        Ok(Self { dir, flake, cfg })
    }
}

pub async fn hack_on(prj: &Project) -> anyhow::Result<()> {
    hack_on_pre_shell(prj).await?;
    hack_on_post_shell(prj).await?;
    Ok(())
}

pub async fn hack_on_pre_shell(prj: &Project) -> anyhow::Result<()> {
    // Run relevant `om health` checks
    tracing::info!("om hack: Running pre-shell checks");
    let health = NixHealth::from_flake(&prj.flake).await?;
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
        for check in check_kind.check(nix_info, Some(&prj.flake)) {
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
    Ok(())
}

pub async fn hack_on_post_shell(prj: &Project) -> anyhow::Result<()> {
    eprintln!();
    print_markdown(&prj.dir, &prj.cfg.readme.get_markdown()).await?;
    Ok(())
}
