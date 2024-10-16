//! Markdown rendering using `mdcat`
use lazy_static::lazy_static;
use pulldown_cmark::{Options, Parser};
use pulldown_cmark_mdcat::{
    resources::FileResourceHandler, Environment, Settings, TerminalProgram, TerminalSize, Theme,
};
use std::{io::Write, path::Path};
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
    print_markdown_to(base_dir, &mut std::io::stderr(), s).await
}

/// Render Markdown into a string to be printed to terminal
pub async fn render_markdown(base_dir: &Path, s: &str) -> anyhow::Result<String> {
    let mut w = Vec::new();
    print_markdown_to(base_dir, &mut w, s).await?;
    let s = String::from_utf8(w)?;
    // A trim is needed to remove unnecessary newlines at end (which can impact for single-line renders)
    Ok(s.trim().to_string())
}

async fn print_markdown_to<W>(base_dir: &Path, w: &mut W, s: &str) -> anyhow::Result<()>
where
    W: Write,
{
    let env = Environment::for_local_directory(&base_dir)?;
    let handler = FileResourceHandler::new(200000);
    let parser = Parser::new_ext(
        s,
        Options::ENABLE_TASKLISTS
            | Options::ENABLE_STRIKETHROUGH
            | Options::ENABLE_TABLES
            | Options::ENABLE_GFM,
    );

    pulldown_cmark_mdcat::push_tty(&SETTINGS, &env, &handler, w, parser)?;

    Ok(())
}
