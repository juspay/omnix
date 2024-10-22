use std::path::Path;

use nix_rs::flake::url::FlakeUrl;
use omnix_common::markdown::print_markdown;

use crate::config::HackConfig;

pub async fn hack_on(dir: &Path) -> anyhow::Result<()> {
    let dir = dir.canonicalize()?;
    let here_flake: FlakeUrl = Into::<FlakeUrl>::into(dir.as_ref());
    let cfg = HackConfig::from_flake(&here_flake).await?;

    // TODO: cachix check

    // Run `om health` foremost
    // TODO: Run with --quiet, possibly using `tracing::subscriber::with_default` (it doesn't work for some reason)
    let checks = omnix_health::run_checks_with(Some(here_flake)).await?;
    let exit_code = omnix_health::NixHealth::print_report_returning_exit_code(&checks).await?;
    if exit_code != 0 {
        anyhow::bail!("Health checks failed");
    }

    eprintln!();
    print_markdown(&dir, &cfg.readme.get_markdown()).await?;

    Ok(())
}
