use std::collections::BTreeMap;

use anyhow::Result;
use nix_rs::{
    command::NixCmd,
    flake::{
        system::System,
        url::{qualified_attr::RootQualifiedAttr, FlakeUrl},
    },
};
use serde::Deserialize;

use crate::cli::BuildConfig;

/// The `nixci` configuration encoded in flake.nix
///
/// Example flake.nix:
/// ```nix
/// {
///   nixci.test = {
///     dir = "./test";
///     overrideInputs = { "mymod" = "."; };
///   };
/// }
#[derive(Debug)]
pub struct Config {
    /// The flake.nix configuration
    pub subflakes: SubflakesConfig,

    pub ref_: ConfigRef,
}

/// A reference into one or all [SubflakesConfig] of some [FlakeUrl]
#[derive(Debug)]
pub struct ConfigRef {
    /// The flake itself
    pub flake_url: FlakeUrl,

    /// The name of the nixci configuration (`omci.<name>`) selected
    pub selected_name: String,

    /// The selected sub-flake if any.
    pub selected_subflake: Option<String>,
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

#[derive(Debug, Deserialize)]
pub struct SubflakesConfig(
    // NB: we use BTreeMap instead of HashMap here so that we always iterate
    // configs in a determinitstic (i.e. asciibetical) order
    pub BTreeMap<String, SubflakeConfig>,
);

impl Default for SubflakesConfig {
    /// Default value contains a single entry for the root flake.
    fn default() -> Self {
        let mut subflakes = BTreeMap::new();
        subflakes.insert("<root>".to_string(), SubflakeConfig::default());
        SubflakesConfig(subflakes)
    }
}

/// Represents a sub-flake look-alike.
///
/// "Look-alike" because its inputs may be partial, thus requiring explicit
/// --override-inputs when evaluating the flake.
#[derive(Debug, Deserialize)]
pub struct SubflakeConfig {
    /// Whether to skip building this subflake
    #[serde(default)]
    pub skip: bool,

    /// Subdirectory in which the flake lives
    pub dir: String,

    /// Inputs to override (via --override-input)
    // NB: we use BTreeMap instead of HashMap here so that we always iterate
    // inputs in a determinitstic (i.e. asciibetical) order
    #[serde(rename = "overrideInputs", default)]
    pub override_inputs: BTreeMap<String, FlakeUrl>,

    /// An optional whitelist of systems to build on (others are ignored)
    pub systems: Option<Vec<System>>,
}

impl Default for SubflakeConfig {
    /// The default `SubflakeConfig` is the root flake.
    fn default() -> Self {
        SubflakeConfig {
            skip: false,
            dir: ".".to_string(),
            override_inputs: BTreeMap::default(),
            systems: None,
        }
    }
}

impl SubflakeConfig {
    pub fn can_build_on(&self, systems: &[System]) -> bool {
        match self.systems.as_ref() {
            Some(systems_whitelist) => systems_whitelist.iter().any(|s| systems.contains(s)),
            None => true,
        }
    }

    /// Return the devour-flake `nix build` arguments for building all the outputs in this
    /// subflake configuration.
    pub fn nix_build_args_for_flake(
        &self,
        build_cfg: &BuildConfig,
        flake_url: &FlakeUrl,
    ) -> Vec<String> {
        std::iter::once(flake_url.sub_flake_url(self.dir.clone()).0)
            .chain(self.override_inputs.iter().flat_map(|(k, v)| {
                [
                    "--override-input".to_string(),
                    // We must prefix the input with "flake" because
                    // devour-flake uses that input name to refer to the user's
                    // flake.
                    format!("flake/{}", k),
                    v.0.to_string(),
                ]
            }))
            .chain([
                "--override-input".to_string(),
                "systems".to_string(),
                build_cfg.systems.0 .0.clone(),
            ])
            .chain(build_cfg.extra_nix_build_args.iter().cloned())
            .collect()
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
