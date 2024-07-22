use std::path::PathBuf;

use clap::Parser;
use flakreate::{
    flake_template::{self, fileop::FileOp, FlakeTemplate},
    nixcmd,
};
use glob::{Pattern, PatternError};
use inquire::Select;
use nix_rs::flake::url::FlakeUrl;

#[derive(Parser, Debug)]
#[clap(author = "Sridhar Ratnakumar", version, about)]
/// Application configuration
struct Args {
    /// whether to be verbose
    #[arg(short = 'v')]
    verbose: bool,

    /// Flake template registry to use
    ///
    /// The flake attribute is treated as a glob pattern to select the
    /// particular template (or subset of templates) to use.
    #[arg(short = 't', default_value = "github:flake-parts/templates")]
    registry: FlakeUrl,

    /// Where to create the template
    #[arg()]
    path: PathBuf,
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

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    if args.verbose {
        println!("DEBUG {args:?}");
    }
    println!(
        "Welcome to flakreate! Let's create your flake template at {:?}:",
        args.path
    );
    let template = FlakeTemplateRegistry::from_url(args.registry.clone())?
        .load_and_select_template()
        .await?;

    // Prompt for template parameters
    let param_values = template.prompt_replacements()?;

    let path = args.path.to_string_lossy();

    // Create the flake templatge
    let template_url = args.registry.with_attr(&template.name);
    println!("$ nix flake new {} -t {}", path, template_url);
    nixcmd()
        .await
        .run_with_args(&["flake", "new", &path, "-t", &template_url.0])
        .await?;

    // Do the actual replacement
    std::env::set_current_dir(args.path)?;
    for replace in param_values {
        FileOp::apply(&replace).await?;
    }
    Ok(())
}
