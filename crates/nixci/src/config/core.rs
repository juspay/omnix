use anyhow::Result;
use nix_rs::{
    command::NixCmd,
    flake::url::{qualified_attr::RootQualifiedAttr, FlakeUrl},
};

use super::{ref_::ConfigRef, subflakes::SubflakesConfig};

/// The nixci configuration encoded in flake.nix
///
/// Example flake.nix:
/// ```nix
/// {
///   om.ci.test = {
///     dir = "./test";
///     overrideInputs = { "mymod" = "."; };
///   };
/// }
#[derive(Debug)]
pub struct Config {
    /// The flake.nix configuration for each subflakes
    pub subflakes: SubflakesConfig,

    /// The reference used by the user to select the configuration
    pub ref_: ConfigRef,
}

impl Config {
    /// Create a `Config` pointed to by this [FlakeUrl]
    ///
    /// Example:
    /// ```text
    /// let url = FlakeUrl("github:srid/haskell-flake#default.dev".to_string());
    /// let cfg = Config::from_flake_url(&url).await?;
    /// ```
    /// along with the config.
    pub async fn from_flake_url(cmd: &NixCmd, url: &FlakeUrl) -> Result<Config> {
        let (mut subflakes, selected_name, rest_attrs) =
            RootQualifiedAttr::new(&["om.ci", "nixci"])
                .eval_flake::<SubflakesConfig>(cmd, url)
                .await?;
        let selected_subflake = rest_attrs.first().cloned();
        if let Some(sub_flake_name) = selected_subflake.clone() {
            if !subflakes.0.contains_key(&sub_flake_name) {
                anyhow::bail!(
                    "Sub-flake '{}' not found in om.ci configuration '{}'",
                    sub_flake_name,
                    url
                )
            }
            for (name, value) in subflakes.0.iter_mut() {
                if name != &sub_flake_name {
                    value.skip = true;
                }
            }
        }
        let ref_ = ConfigRef {
            flake_url: url.without_attr(),
            selected_name,
            selected_subflake,
        };
        let cfg = Config { subflakes, ref_ };
        Ok(cfg)
    }
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
        let cfg = Config::from_flake_url(&NixCmd::default(), url)
            .await
            .unwrap();
        assert_eq!(cfg.ref_.selected_name, "default");
        assert_eq!(cfg.ref_.selected_subflake, Some("dev".to_string()));
        assert_eq!(cfg.subflakes.0.len(), 7);
    }
}
