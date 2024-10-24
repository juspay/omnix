use std::path::PathBuf;

use clap::Parser;

/// Prepare to hack on a flake project
#[derive(Parser, Debug)]
pub struct HackCommand {
    /// Directory of the project
    #[arg(name = "DIR", default_value = ".")]
    dir: PathBuf,

    // TODO: Implement these options
    // om hack --mode-before
    // om hack --mode-after-success
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

impl HackCommand {
    pub async fn run(&self) -> anyhow::Result<()> {
        let prj = omnix_hack::core::Project::new(&self.dir).await?;
        match self.stage {
            Some(Stage::PreShell) => omnix_hack::core::hack_on_pre_shell(&prj).await?,
            Some(Stage::PostShell) => omnix_hack::core::hack_on_post_shell(&prj).await?,
            None => omnix_hack::core::hack_on(&prj).await?,
        }
        Ok(())
    }
}
