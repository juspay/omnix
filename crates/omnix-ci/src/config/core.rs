//! The top-level configuration of omnix-ci, as defined in flake.nix

#[cfg(test)]
mod tests {
    use nix_rs::{command::NixCmd, flake::url::FlakeUrl};
    use omnix_common::config::OmConfig;

    use crate::config::subflakes::SubflakesConfig;

    #[tokio::test]
    async fn test_config_loading() {
        // Testing this flake:
        // https://github.com/srid/haskell-flake/blob/76214cf8b0d77ed763d1f093ddce16febaf07365/flake.nix#L15-L67
        let url = &FlakeUrl(
            "github:srid/haskell-flake/76214cf8b0d77ed763d1f093ddce16febaf07365#default.dev"
                .to_string(),
        );
        let cfg = OmConfig::get(&NixCmd::default(), url).await.unwrap();
        let (config, attrs) = cfg.get_sub_config_under::<SubflakesConfig>("ci").unwrap();
        assert_eq!(attrs, &["dev"]);
        // assert_eq!(cfg.selected_subconfig, Some("dev".to_string()));
        assert_eq!(config.0.len(), 7);
    }
}
