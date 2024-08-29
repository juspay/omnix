//! Manage omnix configuration in flake.nix

use nix_rs::{
    command::NixCmd,
    flake::url::{
        attr::FlakeAttr,
        qualified_attr::{QualifiedAttrError, RootQualifiedAttr},
        FlakeUrl,
    },
};
use serde::de::DeserializeOwned;

/// Reference to some Omnix configuration of type `T` in a flake
///
/// For example, CI configuration at `om.ci.default` is captured by the `T` type.
///
/// TODO: This type needs to support `om.templates` style configuration as well, where there is no key'ed config (such as "default").
#[derive(Debug)]
pub struct OmConfig<T> {
    /// The flake URL used to load this configuration
    pub flake_url: FlakeUrl,

    /// The name of the configuration (`om.??.<name>`) selected
    pub selected_name: String,

    /// The selected sub-config name if any (`om.??.<name>.<subconfig>`)
    pub selected_subconfig: Option<String>,

    /// The whole `om.??.<name>` configuration parsed as `T`
    pub selected_config: T,
}

impl<T> OmConfig<T> {
    /// Read the Om configuration from the flake URL
    pub async fn from_flake_url<S>(
        cmd: &NixCmd,
        url: &FlakeUrl,
        k: &[S],
    ) -> Result<OmConfig<T>, QualifiedAttrError>
    where
        S: AsRef<str>,
        T: Default + DeserializeOwned,
    {
        let (selected_config, selected_name, rest_attrs) =
            RootQualifiedAttr::new(k).eval_flake::<T>(cmd, url).await?;
        let selected_subconfig = rest_attrs.first().cloned();
        Ok(OmConfig {
            flake_url: url.without_attr(),
            selected_name,
            selected_subconfig,
            selected_config,
        })
    }

    /// Return the non-default attribute that selected this sub-configuration.
    pub fn get_attr(&self) -> FlakeAttr {
        if let Some(subconfig) = &self.selected_subconfig {
            FlakeAttr::new(&format!("{}.{}", self.selected_name, subconfig))
        } else if self.selected_name == "default" {
            FlakeAttr::none()
        } else {
            FlakeAttr::new(&self.selected_name)
        }
    }
}
