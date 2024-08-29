//! The top-level configuration of nixci, as defined in flake.nix
use anyhow::Result;
use nix_rs::{command::NixCmd, flake::url::FlakeUrl};
use omnix_common::config::OmConfig;

use super::subflakes::SubflakesConfig;

/// Create a `Config` pointed to by this [FlakeUrl]
///
/// Example:
/// ```text
/// let url = FlakeUrl("github:srid/haskell-flake#default.dev".to_string());
/// let cfg = Config::from_flake_url(&url).await?;
/// ```
/// along with the config.
pub async fn ci_config_from_flake_url(
    cmd: &NixCmd,
    url: &FlakeUrl,
) -> Result<OmConfig<SubflakesConfig>> {
    let mut om_config = omnix_common::config::OmConfig::<SubflakesConfig>::from_flake_url(
        cmd,
        url,
        &["om.ci", "nixci"],
    )
    .await?;
    if let Some(sub_flake_name) = om_config.selected_subconfig.clone() {
        if !om_config.selected_config.0.contains_key(&sub_flake_name) {
            anyhow::bail!(
                "Sub-flake '{}' not found in om.ci configuration '{}'",
                sub_flake_name,
                url
            )
        }
        for (name, value) in om_config.selected_config.0.iter_mut() {
            if name != &sub_flake_name {
                value.skip = true;
            }
        }
    }
    Ok(om_config)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_config_loading() {
        // Testing this flake:
        // https://github.com/srid/haskell-flake/blob/76214cf8b0d77ed763d1f093ddce16febaf07365/flake.nix#L15-L67
        let url = &FlakeUrl(
            "github:srid/haskell-flake/76214cf8b0d77ed763d1f093ddce16febaf07365#default.dev"
                .to_string(),
        );
        let cfg = ci_config_from_flake_url(&NixCmd::default(), url)
            .await
            .unwrap();
        assert_eq!(cfg.selected_name, "default");
        assert_eq!(cfg.selected_subconfig, Some("dev".to_string()));
        assert_eq!(cfg.selected_config.0.len(), 7);
    }
}
