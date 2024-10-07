use std::fmt::{self, Display, Formatter};

use colored::Colorize;
use nix_rs::{
    command::NixCmd,
    flake::{command::FlakeOptions, url::FlakeUrl},
};
use omnix_common::config::OmConfig;

use crate::template::Template;

/// A named [Template] associated with a [FlakeUrl]
#[derive(Debug, Clone)]
pub(crate) struct FlakeTemplate<'a> {
    pub(crate) flake: &'a FlakeUrl,
    pub(crate) template_name: String,
    pub(crate) template: Template,
}

impl<'a> Display for FlakeTemplate<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:<15} {} {}",
            self.template_name,
            format!("[{}]", self.flake).dimmed(),
            self.template
                .template
                .description
                .as_ref()
                .unwrap_or(&"".to_string())
        )
    }
}

/// Load templates from the given flake
pub(crate) async fn load_templates<'a>(url: &FlakeUrl) -> anyhow::Result<Vec<FlakeTemplate>> {
    let _opts = FlakeOptions {
        refresh: true,
        ..Default::default()
    };
    let v = OmConfig::<Template>::from_flake_url(NixCmd::get().await, url, &["om.templates"])
        .await?
        .config;
    Ok(v.into_iter()
        .map(|(k, v)| FlakeTemplate {
            flake: url,
            template_name: k,
            template: v,
        })
        .collect())
}
