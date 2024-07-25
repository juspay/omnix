use std::collections::BTreeMap;

use anyhow::Result;
use nix_rs::{
    command::NixCmd,
    flake::{eval::nix_eval_attr_json, system::System, url::FlakeUrl},
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
// NB: we use BTreeMap instead of HashMap here so that we always iterate
// configs in a determinitstic (i.e. asciibetical) order
#[derive(Debug)]
pub struct Config {
    /// The flake.nix configuration
    pub subflakes: Subflakes,

    /// The URL to the flake containing this configuration
    pub flake_url: FlakeUrl,

    /// Configuration name (nixci.<name>)
    pub name: String,

    /// Selected sub-flake if any.
    ///
    /// Must be a key in `subflakes`.
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
        let (flake_url, attr) = url.split_attr();
        let nested_attr = attr.as_list();
        let (name, selected_subflake) = match nested_attr.as_slice() {
            [] => ("default".to_string(), None),
            [name] => (name.clone(), None),
            [name, sub_flake] => (name.clone(), Some(sub_flake.to_string())),
            _ => anyhow::bail!("Invalid flake URL (too many nested attr): {}", flake_url.0),
        };
        let nixci_url = FlakeUrl(format!("{}#nixci.{}", flake_url.0, name));
        let subflakes = nix_eval_attr_json::<Subflakes>(cmd, &nixci_url, attr.is_none()).await?;
        if let Some(sub_flake_name) = selected_subflake.clone() {
            if !subflakes.0.contains_key(&sub_flake_name) {
                anyhow::bail!(
                    "Sub-flake '{}' not found in nixci configuration '{}'",
                    sub_flake_name,
                    nixci_url
                )
            }
        }
        let cfg = Config {
            subflakes,
            flake_url,
            name,
            selected_subflake,
        };
        Ok(cfg)
    }
}

#[derive(Debug, Deserialize)]
pub struct Subflakes(pub BTreeMap<String, SubFlakish>);

impl Default for Subflakes {
    /// Default value contains a single entry for the root flake.
    fn default() -> Self {
        let mut subflakes = BTreeMap::new();
        subflakes.insert("<root>".to_string(), SubFlakish::default());
        Subflakes(subflakes)
    }
}

/// Represents a sub-flake look-alike.
///
/// "Look-alike" because its inputs may be partial, thus requiring explicit
/// --override-inputs when evaluating the flake.
#[derive(Debug, Deserialize)]
pub struct SubFlakish {
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

impl Default for SubFlakish {
    /// The default `SubFlakish` is the root flake.
    fn default() -> Self {
        SubFlakish {
            dir: ".".to_string(),
            override_inputs: BTreeMap::default(),
            systems: None,
        }
    }
}

impl SubFlakish {
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
#[cfg(feature = "integration_test")]
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
        assert_eq!(cfg.name, "default");
        assert_eq!(cfg.selected_subflake, Some("dev".to_string()));
        assert_eq!(cfg.subflakes.0.len(), 7);
    }
}
