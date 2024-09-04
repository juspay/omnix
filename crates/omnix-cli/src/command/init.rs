use std::path::PathBuf;

use clap::Parser;

/// Initialize a new flake project
#[derive(Parser, Debug)]
pub struct InitCommand {
    /// Where to create the template
    #[arg(short = 'o', long = "output")]
    path: PathBuf,

    /// The name of the template to use
    ///
    /// If not passed, the user will presented with a list of templates to choose from.
    ///
    /// In future, this will support arbitrary URLs. For now, we only support builtin templates.
    template: Option<String>,
}

impl InitCommand {
    pub async fn run(&self) -> anyhow::Result<()> {
        tracing::warn!("\n  !! WARNING: `om init` is still under development !!\n");
        omnix_init::core::initialize_template(&self.path, self.template.clone()).await
    }
}
