use nix_health::{traits::Checkable, NixHealth};
use nix_rs::{flake::url::FlakeUrl, info::NixInfo};

pub async fn check_nix_version(flake_url: &FlakeUrl, nix_info: &NixInfo) -> anyhow::Result<()> {
    let nix_health = NixHealth::from_flake(flake_url).await?;
    let checks = nix_health.nix_version.check(nix_info, Some(flake_url));
    let exit_code = NixHealth::print_report_returning_exit_code(&checks);

    if exit_code != 0 {
        std::process::exit(exit_code);
    }
    Ok(())
}
