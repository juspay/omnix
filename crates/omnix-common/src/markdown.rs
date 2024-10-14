//! Markdown rendering using `mdcat`
use lazy_static::lazy_static;
use pulldown_cmark::{Options, Parser};
use pulldown_cmark_mdcat::{
    resources::FileResourceHandler, Environment, Settings, TerminalProgram, TerminalSize, Theme,
};
use std::path::Path;
use syntect::parsing::SyntaxSet;

lazy_static! {
    static ref SYNTAX_SET: SyntaxSet = SyntaxSet::load_defaults_newlines();

    /// Global settings for rendering markdown
    pub static ref SETTINGS: Settings<'static> = Settings {
        terminal_capabilities: TerminalProgram::detect().capabilities(),
        terminal_size: TerminalSize::from_terminal().unwrap_or_default(),
        theme: Theme::default(),
        syntax_set: &SYNTAX_SET,
    };
}

/// Print Markdown using `mdcat` to STDERR
pub async fn print_markdown(base_dir: &Path, s: &str) -> anyhow::Result<()> {
    let env = Environment::for_local_directory(&base_dir)?;
    let handler = FileResourceHandler::new(200000);
    let parser = Parser::new_ext(
        s,
        Options::ENABLE_TASKLISTS
            | Options::ENABLE_STRIKETHROUGH
            | Options::ENABLE_TABLES
            | Options::ENABLE_GFM,
    );

    pulldown_cmark_mdcat::push_tty(&SETTINGS, &env, &handler, &mut std::io::stderr(), parser)?;

    Ok(())
}
