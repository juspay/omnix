use clap::Parser;
use nix_rs::flake::url::FlakeUrl;
use omnix_common::config::OmConfig;

/// Prepare to develop on a flake project
#[derive(Parser, Debug)]
pub struct DevelopCommand {
    /// Directory of the project
    #[arg(name = "DIR", default_value = ".")]
    flake_shell: FlakeUrl,

    /// The stage to run in. If not provided, runs all stages.
    #[arg(long, value_enum)]
    stage: Option<Stage>,
}

/// The stage to run in
#[derive(clap::ValueEnum, Debug, Clone)]
enum Stage {
    /// Stage before Nix shell is invoked.
    PreShell,

    /// Stage after Nix shell is successfully invoked.
    PostShell,
}

impl DevelopCommand {
    pub async fn run(&self) -> anyhow::Result<()> {
        let flake = self.flake_shell.without_attr();

        let om_config = OmConfig::get(&flake).await?;

        tracing::info!("⌨️  Preparing to develop project: {:}", &flake);
        let prj = omnix_develop::core::Project::new(flake, om_config).await?;
        match self.stage {
            Some(Stage::PreShell) => omnix_develop::core::develop_on_pre_shell(&prj).await?,
            Some(Stage::PostShell) => omnix_develop::core::develop_on_post_shell(&prj).await?,
            None => omnix_develop::core::develop_on(&prj).await?,
        }
        Ok(())
    }
}
