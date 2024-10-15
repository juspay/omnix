use std::path::Path;

use nix_rs::flake::url::FlakeUrl;
use omnix_common::markdown::print_markdown;

use crate::config::HackConfig;

pub async fn hack_on() -> anyhow::Result<()> {
    let here_flake: FlakeUrl = Into::<FlakeUrl>::into(Path::new("."));
    let cfg = HackConfig::from_flake(&here_flake).await?;

    // TODO: cachix check
    // TODO: `om health`

    let pwd = std::env::current_dir()?;
    eprintln!();
    print_markdown(&pwd, &cfg.readme.get_markdown()).await?;

    Ok(())
}
