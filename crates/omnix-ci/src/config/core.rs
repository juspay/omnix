//! The top-level configuration of omnix-ci, as defined in flake.nix

#[cfg(test)]
mod tests {
    use nix_rs::flake::url::FlakeUrl;
    use omnix_common::config::OmConfig;

    use crate::config::subflakes::SubflakesConfig;

    #[tokio::test]
    async fn test_config_loading() {
        // Testing this flake:
        // https://github.com/srid/haskell-flake/blob/c60351652c71ebeb5dd237f7da874412a7a96970/flake.nix#L30-L95
        let url = &FlakeUrl(
            "github:srid/haskell-flake/c60351652c71ebeb5dd237f7da874412a7a96970#default.dev"
                .to_string(),
        );
        let cfg = OmConfig::get(url).await.unwrap();
        let (config, attrs) = cfg.get_sub_config_under::<SubflakesConfig>("ci").unwrap();
        assert_eq!(attrs, &["dev"]);
        // assert_eq!(cfg.selected_subconfig, Some("dev".to_string()));
        assert_eq!(config.0.len(), 9);
    }
}
