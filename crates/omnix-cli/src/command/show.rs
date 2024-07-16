use std::collections::BTreeMap;

use anyhow::Context;
use clap::Parser;
use colored::Colorize;
use nix_rs::{
    command::NixCmd,
    flake::{outputs::Val, url::FlakeUrl, Flake},
};
use tabled::{settings::Style, Table, Tabled};

#[derive(Parser, Debug)]
pub struct ShowConfig {
    /// The flake url to show outputs for
    pub flake_url: FlakeUrl,
}

/// Tabular output for a given field in [nix_rs::flake::schema::FlakeSchema]
pub struct FlakeOutputTable {
    /// Rows of the table
    pub rows: Vec<Row>,
    /// Title of the table
    pub title: String,
    /// Command to run the outputs in the `name` column
    pub command: String,
}

impl FlakeOutputTable {
    /// Convert the table to a [Table] struct
    fn to_tabled(&self) -> Table {
        let mut table = Table::new(&self.rows);
        table.with(Style::modern());
        table
    }
    /// Print the table to stdout
    pub fn print(&self) {
        if self.rows.is_empty() {
            return;
        }
        println!("{}", self.title.blue().bold());
        println!();
        println!("Run: {}", self.command.green().bold());

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
                description: val.description.unwrap_or_else(|| "N/A".to_string()),
            })
            .collect()
    }
}

impl ShowConfig {
    pub async fn run(&self) -> anyhow::Result<()> {
        let flake = Flake::from_nix(&NixCmd::default(), self.flake_url.clone())
            .await
            .with_context(|| "Unable to fetch flake")?;

        FlakeOutputTable {
            rows: Row::vec_from_btreemap(flake.schema.packages),
            title: "📦 Packages".to_string(),
            command: "nix build .#<name>".to_string(),
        }
        .print();

        FlakeOutputTable {
            rows: Row::vec_from_btreemap(flake.schema.devshells),
            title: "🐚 Devshells".to_string(),
            command: "nix develop .#<name>".to_string(),
        }
        .print();

        FlakeOutputTable {
            rows: Row::vec_from_btreemap(flake.schema.apps),
            title: "🚀 Apps".to_string(),
            command: "nix run .#<name>".to_string(),
        }
        .print();

        FlakeOutputTable {
            rows: Row::vec_from_btreemap(flake.schema.checks),
            title: "🔍 Checks".to_string(),
            command: "nix flake check".to_string(),
        }
        .print();

        Ok(())
    }
}
