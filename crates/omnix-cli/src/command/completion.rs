use crate::Args;
use clap::CommandFactory;
use clap_complete::generate;
use clap_complete::Shell;

pub fn generate_completion(shell: Shell) -> anyhow::Result<()> {
    let mut cli = Args::command();
    generate(shell, &mut cli, "om", &mut std::io::stdout());
    Ok(())
}
