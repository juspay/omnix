use anyhow::Context;
use std::path::{Path, PathBuf};

use nix_rs::{flake::url::FlakeUrl, info::NixInfo};
use omnix_common::markdown::print_markdown;
use omnix_health::{check::caches::CachixCache, traits::Checkable, NixHealth};

use crate::config::DevelopConfig;

/// A project that an be developed on locally.
pub struct Project {
    /// The directory of the project.
    pub dir: PathBuf,
    /// [FlakeUrl] corresponding to the project.
    pub flake: FlakeUrl,
    /// The develop configuration
    pub cfg: DevelopConfig,
}

impl Project {
    pub async fn new(dir: &Path) -> anyhow::Result<Self> {
        let dir = dir.canonicalize()?;
        let flake: FlakeUrl = Into::<FlakeUrl>::into(dir.as_ref());
        let cfg = DevelopConfig::from_flake(&flake).await?;
        Ok(Self { dir, flake, cfg })
    }
}

pub async fn develop_on(prj: &Project) -> anyhow::Result<()> {
    develop_on_pre_shell(prj).await?;
    develop_on_post_shell(prj).await?;
    Ok(())
}

pub async fn develop_on_pre_shell(prj: &Project) -> anyhow::Result<()> {
    // Run relevant `om health` checks
    let health = NixHealth::from_flake(&prj.flake).await?;
    let nix_info = NixInfo::get()
        .await
        .as_ref()
        .with_context(|| "Unable to gather nix info")?;

    let mut relevant_checks: Vec<&'_ dyn Checkable> =
        vec![&health.nix_version, &health.rosetta, &health.max_jobs];
    if !health.caches.required.is_empty() {
        relevant_checks.push(&health.trusted_users);
    };

    // Run cache related checks, and try to resolve it automatically using `cachix use` as appropriate
    if !health.caches.required.is_empty() {
        let missing = health.caches.get_missing_caches(nix_info);
        let (missing_cachix, missing_other) = parse_many(&missing, CachixCache::from_url);
        for cachix_cache in &missing_cachix {
            tracing::info!("🐦 Running `cachix use` for {}", cachix_cache.0);
            cachix_cache.cachix_use().await?;
        }
        if !missing_other.is_empty() {
            // We cannot add these caches automatically, so defer to `om health`
            relevant_checks.push(&health.caches);
        };
        // TODO: Re-calculate NixInfo since our nix.conf has changed (due to `cachix use`)
        // To better implement this, we need a mutable database of NixInfo, NixConfig, etc. OnceCell is not sufficient
    };

    for check_kind in relevant_checks.into_iter() {
        for check in check_kind.check(nix_info, Some(&prj.flake)) {
            if !check.result.green() {
                check.tracing_log().await?;
                if !check.result.green() && check.required {
                    anyhow::bail!("ERROR: Your Nix invironment is not properly setup. See suggestions above, or run `om health` for details.");
                };
            };
        }
    }

    tracing::info!("✅ Nix environment is healthy.");

    Ok(())
}

pub async fn develop_on_post_shell(prj: &Project) -> anyhow::Result<()> {
    eprintln!();
    print_markdown(&prj.dir, prj.cfg.readme.get_markdown()).await?;
    Ok(())
}

/// Parse all items using the given parse function
fn parse_many<'a, T, Q, F>(vec: &'a [T], f: F) -> (Vec<Q>, Vec<&'a T>)
where
    F: Fn(&T) -> Option<Q>,
{
    let mut successes: Vec<Q> = Vec::new();
    let mut failures: Vec<&'a T> = Vec::new();

    for item in vec {
        match f(item) {
            Some(transformed) => successes.push(transformed),
            None => failures.push(item),
        }
    }

    (successes, failures)
}
