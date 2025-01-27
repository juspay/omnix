use clap::CommandFactory;
use clap::Parser;
use clap_complete::generate;

/// Generates shell completion scripts
#[derive(Parser, Debug)]
pub struct CompletionCommand {
    #[arg(value_enum)]
    shell: Shell2,
}

impl CompletionCommand {
    pub fn run(&self) -> anyhow::Result<()> {
        generate_completion(self.shell);
        Ok(())
    }
}

/// Like `clap::Shell`, but with an additional variant for Nushell
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, clap::ValueEnum)]
#[non_exhaustive]
enum Shell2 {
    Bash,
    Elvish,
    Fish,
    PowerShell,
    Zsh,
    Nushell,
}

impl Shell2 {
    fn nushell_or(self) -> Result<clap_complete::Shell, clap_complete_nushell::Nushell> {
        match self {
            Shell2::Nushell => Err(clap_complete_nushell::Nushell),
            Shell2::Bash => Ok(clap_complete::Shell::Bash),
            Shell2::Elvish => Ok(clap_complete::Shell::Elvish),
            Shell2::Fish => Ok(clap_complete::Shell::Fish),
            Shell2::PowerShell => Ok(clap_complete::Shell::PowerShell),
            Shell2::Zsh => Ok(clap_complete::Shell::Zsh),
        }
    }
}

fn generate_completion(shell: Shell2) {
    let cmd = &mut crate::args::Args::command();
    let out = &mut std::io::stdout();
    match shell.nushell_or() {
        Ok(shell) => generate(shell, cmd, "om", out),
        Err(shell) => generate(shell, cmd, "om", out),
    }
}
