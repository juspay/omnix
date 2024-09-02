use std::{collections::BTreeMap, io::IsTerminal};

use anyhow::Context;
use clap::Parser;
use colored::Colorize;
use nix_rs::{
    command::NixCmd,
    config::NixConfig,
    flake::{outputs::Val, url::FlakeUrl, Flake},
};
use tabled::{
    settings::{location::ByColumnName, Color, Modify, Style},
    Table, Tabled,
};

/// Inspect the outputs of a flake
#[derive(Parser, Debug)]
pub struct ShowCommand {
    /// The flake to show outputs for
    #[arg(name = "FLAKE")]
    pub flake_url: FlakeUrl,
}

/// Tabular representation of a set of flake outputs (eg: `packages.*`)
pub struct FlakeOutputTable {
    /// Rows of the table
    pub rows: Vec<Row>,
    /// Title of the table
    pub title: String,
    /// Command to run the outputs in the `name` column
    pub command: Option<String>,
}

impl FlakeOutputTable {
    /// Convert the table to a [Table] struct
    fn to_tabled(&self) -> Table {
        let mut table = Table::new(&self.rows);
        table.with(Style::rounded());
        if std::io::stdout().is_terminal() {
            table.with(Modify::new(ByColumnName::new("name")).with(Color::BOLD));
        };
        table
    }

    /// Print the table to stdout
    pub fn print(&self) {
        if self.rows.is_empty() {
            return;
        }
        print!("{}", self.title.blue().bold());

        if let Some(command) = &self.command {
            println!(" ({})", command.green().bold());
        } else {
            // To ensure the table name and the table are on separate lines
            println!();
        }

        println!("{}", self.to_tabled());
        println!();
    }
}

/// Row in a [FlakeOutputTable]
#[derive(Tabled)]
pub struct Row {
    /// Name of the output
    pub name: String,
    /// Description of the output
    pub description: String,
}

impl Row {
    /// Convert a [BTreeMap] to a vector of [Row]s
    pub fn vec_from_btreemap(map: BTreeMap<String, Val>) -> Vec<Row> {
        map.into_iter()
            .map(|(name, val)| Row {
                name,
                description: val.short_description.unwrap_or("N/A".to_owned()),
            })
            .collect()
    }
}

impl ShowCommand {
    pub async fn run(&self) -> anyhow::Result<()> {
        let nix_cmd = NixCmd::get().await;
        let nix_config = NixConfig::get().await.as_ref()?;
        let flake = Flake::from_nix(nix_cmd, nix_config, self.flake_url.clone())
            .await
            .with_context(|| "Unable to fetch flake")?;

        FlakeOutputTable {
            rows: Row::vec_from_btreemap(flake.schema.packages),
            title: "📦 Packages".to_string(),
            command: Some(format!("nix build {}#<name>", self.flake_url)),
        }
        .print();

        FlakeOutputTable {
            rows: Row::vec_from_btreemap(flake.schema.devshells),
            title: "🐚 Devshells".to_string(),
            command: Some(format!("nix develop {}#<name>", self.flake_url)),
        }
        .print();

        FlakeOutputTable {
            rows: Row::vec_from_btreemap(flake.schema.apps),
            title: "🚀 Apps".to_string(),
            command: Some(format!("nix run {}#<name>", self.flake_url)),
        }
        .print();

        FlakeOutputTable {
            rows: Row::vec_from_btreemap(flake.schema.checks),
            title: "🔍 Checks".to_string(),
            command: Some("nix flake check".to_string()),
        }
        .print();

        FlakeOutputTable {
            rows: Row::vec_from_btreemap(flake.schema.nixos_configurations),
            title: "🐧 NixOS Configurations".to_string(),
            command: Some(format!(
                "nixos-rebuild switch --flake {}#<name>",
                self.flake_url
            )),
        }
        .print();

        FlakeOutputTable {
            rows: Row::vec_from_btreemap(flake.schema.darwin_configurations),
            title: "🍏 Darwin Configurations".to_string(),
            command: Some(format!(
                "darwin-rebuild switch --flake {}#<name>",
                self.flake_url
            )),
        }
        .print();

        FlakeOutputTable {
            rows: Row::vec_from_btreemap(flake.schema.nixos_modules),
            title: "🔧 NixOS Modules".to_string(),
            // TODO: Command should be optional
            command: None,
        }
        .print();

        FlakeOutputTable {
            rows: Row::vec_from_btreemap(flake.schema.docker_images),
            title: "🐳 Docker Images".to_string(),
            // TODO: Try if the below command works
            command: Some(format!("nix build {}#dockerImages.<name>", self.flake_url)),
        }
        .print();

        FlakeOutputTable {
            rows: Row::vec_from_btreemap(flake.schema.overlays),
            title: "🎨 Overlays".to_string(),
            command: None,
        }
        .print();

        FlakeOutputTable {
            rows: Row::vec_from_btreemap(flake.schema.templates),
            title: "📝 Templates".to_string(),
            command: Some(format!("nix flake init -t {}#<name>", self.flake_url)),
        }
        .print();

        FlakeOutputTable {
            rows: Row::vec_from_btreemap(flake.schema.schemas),
            title: "📜 Schemas".to_string(),
            command: None,
        }
        .print();

        Ok(())
    }
}
