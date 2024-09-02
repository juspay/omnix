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
pub async fn ci_config_from_flake_outputs(
    cmd: &NixCmd,
    url: &FlakeUrl,
) -> Result<OmConfig<SubflakesConfig>> {
    let v = omnix_common::config::OmConfig::<SubflakesConfig>::from_flake_outputs(
        cmd,
        url,
        &["om.ci", "nixci"],
    )
    .await?;
    Ok(v)
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
        let cfg = ci_config_from_flake_outputs(&NixCmd::default(), url)
            .await
            .unwrap();
        let (config, attrs) = cfg.get_referenced().unwrap();
        assert_eq!(attrs, &["dev"]);
        // assert_eq!(cfg.selected_subconfig, Some("dev".to_string()));
        assert_eq!(config.0.len(), 7);
    }
}
