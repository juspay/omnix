#![feature(lazy_cell)]
use std::env::var;
use std::path::PathBuf;
use std::sync::LazyLock;

use clap::Parser;
use flake_template::fileop::FileOp;
use flake_template::FlakeTemplate;
use glob::{Pattern, PatternError};
use inquire::Select;
use nix_rs::command::NixCmd;
use nix_rs::flake::url::FlakeUrl;
use tokio::sync::OnceCell;

pub mod flake_template;

static NIXCMD: OnceCell<NixCmd> = OnceCell::const_new();

/// TODO: Can we normalize this across omnix-cli?
pub async fn nixcmd() -> &'static NixCmd {
    NIXCMD
        .get_or_init(|| async { NixCmd::default().with_flakes().await.unwrap() })
        .await
}

static REGISTRY: LazyLock<FlakeUrl> =
    LazyLock::new(|| PathBuf::from(var("FLAKREATE_REGISTRY").unwrap()).into());

#[derive(Parser, Debug)]
#[clap(author = "Sridhar Ratnakumar", version, about)]
/// Application configuration
pub struct Args {
    /// whether to be verbose
    #[arg(short = 'v')]
    pub verbose: bool,

    /// Flake template registry to use
    ///
    /// The flake attribute is treated as a glob pattern to select the
    /// particular template (or subset of templates) to use.
    #[arg(short = 't', default_value_t = REGISTRY.clone())]
    pub registry: FlakeUrl,

    /// Where to create the template
    #[arg()]
    pub path: PathBuf,
}

struct FlakeTemplateRegistry {
    pub flake_url: FlakeUrl,
    pub filter: Option<Pattern>,
}

impl FlakeTemplateRegistry {
    pub fn from_url(url: FlakeUrl) -> Result<Self, PatternError> {
        let (base, attr) = url.split_attr();
        Ok(FlakeTemplateRegistry {
            flake_url: base,
            filter: if attr.is_none() {
                None
            } else {
                Some(Pattern::new(&attr.get_name())?)
            },
        })
    }

    pub async fn load_and_select_template(&self) -> anyhow::Result<FlakeTemplate> {
        let term = console::Term::stdout();
        term.write_line(format!("Loading registry {}...", self.flake_url).as_str())?;
        let templates = flake_template::fetch(&self.flake_url).await?;
        term.clear_last_lines(1)?;
        println!("Loaded registry: {}", self.flake_url);
        // TODO: avoid duplicates (aliases)
        let filtered_templates = templates
            .iter()
            .filter(|template| {
                self.filter
                    .as_ref()
                    .map_or(true, |filter| filter.matches(&template.name))
            })
            .collect::<Vec<_>>();
        let template = if filtered_templates.len() == 1 {
            filtered_templates[0]
        } else {
            Select::new("Select a template", filtered_templates)
                .with_help_message("Choose a flake template to use")
                .prompt()?
        };
        println!("Selected template: {}", template);
        Ok(template.clone())
    }
}

pub async fn flakreate(registry: FlakeUrl, path: PathBuf) -> anyhow::Result<()> {
    println!(
        "Welcome to flakreate! Let's create your flake template at {:?}:",
        path
    );
    let template = FlakeTemplateRegistry::from_url(registry.clone())?
        .load_and_select_template()
        .await?;

    // Prompt for template parameters
    let param_values = template.prompt_replacements()?;

    let path = path.to_string_lossy();

    // Create the flake templatge
    let template_url = registry.with_attr(&template.name);
    println!("$ nix flake new {} -t {}", path, template_url);
    nixcmd()
        .await
        .run_with_args(&["flake", "new", &path, "-t", &template_url.0])
        .await?;

    // Do the actual replacement
    std::env::set_current_dir(&*path)?;
    for replace in param_values {
        FileOp::apply(&replace).await?;
    }
    Ok(())
}
