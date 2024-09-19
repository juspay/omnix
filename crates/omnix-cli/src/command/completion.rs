use clap::CommandFactory;
use clap::Parser;
use clap_complete::generate;
use clap_complete::Shell;

/// Generates shell completion scripts
#[derive(Parser, Debug)]
pub struct CompletionCommand {
    #[arg(value_enum)]
    shell: clap_complete::Shell,
}

impl CompletionCommand {
    pub fn run(&self) -> anyhow::Result<()> {
        generate_completion(self.shell);
        Ok(())
    }
}

fn generate_completion(shell: Shell) {
    generate(
        shell,
        &mut crate::args::Args::command(),
        "om",
        &mut std::io::stdout(),
    )
}
