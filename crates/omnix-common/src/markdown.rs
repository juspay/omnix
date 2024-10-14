//! Markdown rendering using `mdcat`
use pulldown_cmark::{Options, Parser};
use pulldown_cmark_mdcat::{
    resources::FileResourceHandler, Environment, Settings, TerminalProgram, TerminalSize, Theme,
};
use std::path::Path;
use syntect::parsing::SyntaxSet;

/// Print Markdown using `mdcat` to STDERR
pub async fn print_markdown(base_dir: &Path, s: &str) -> anyhow::Result<()> {
    // Create a new environment for rendering
    let env = Environment::for_local_directory(&base_dir)?;

    // Create default settings
    let settings = Settings {
        terminal_capabilities: TerminalProgram::detect().capabilities(),
        terminal_size: TerminalSize::default(),
        theme: Theme::default(),
        syntax_set: &SyntaxSet::default(),
    };

    let handler = FileResourceHandler::new(200000);

    let parser = Parser::new_ext(
        s,
        Options::ENABLE_TASKLISTS
            | Options::ENABLE_STRIKETHROUGH
            | Options::ENABLE_TABLES
            | Options::ENABLE_GFM,
    );

    let mut sink = std::io::stderr();
    pulldown_cmark_mdcat::push_tty(&settings, &env, &handler, &mut sink, parser)?;

    Ok(())
}
