use std::collections::BTreeMap;

use anyhow::Context;
use clap::Parser;
use colored::Colorize;
use nix_rs::{
    command::NixCmd,
    flake::{outputs::Val, url::FlakeUrl, Flake},
};
use tabled::{
    settings::{
        style::{HorizontalLine, VerticalLine},
        Style,
    },
    Table, Tabled,
};

/// Inspect a flake
#[derive(Parser, Debug)]
pub struct ShowConfig {
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
    pub command: String,
}

impl FlakeOutputTable {
    /// Convert the table to a [Table] struct
    fn to_tabled(&self) -> Table {
        let mut table = Table::new(&self.rows);
        table.with(
            Style::modern()
                .horizontals([(1, HorizontalLine::inherit(Style::modern()).horizontal('═'))])
                .verticals([(1, VerticalLine::inherit(Style::modern()))])
                .remove_horizontal()
                .remove_vertical()
                .remove_left()
                .remove_right(),
        );
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
        let flake = Flake::from_nix(NixCmd::get().await, self.flake_url.clone())
            .await
            .with_context(|| "Unable to fetch flake")?;

        FlakeOutputTable {
            rows: Row::vec_from_btreemap(flake.schema.packages),
            title: "📦 Packages".to_string(),
            command: format!("nix build {}#<name>", self.flake_url),
        }
        .print();

        FlakeOutputTable {
            rows: Row::vec_from_btreemap(flake.schema.devshells),
            title: "🐚 Devshells".to_string(),
            command: format!("nix develop {}#<name>", self.flake_url),
        }
        .print();

        FlakeOutputTable {
            rows: Row::vec_from_btreemap(flake.schema.apps),
            title: "🚀 Apps".to_string(),
            command: format!("nix run {}#<name>", self.flake_url),
        }
        .print();

        FlakeOutputTable {
            rows: Row::vec_from_btreemap(flake.schema.checks),
            title: "🔍 Checks".to_string(),
            command: "nix flake check".to_string(),
        }
        .print();

        FlakeOutputTable {
            rows: Row::vec_from_btreemap(flake.schema.nixos_configurations),
            title: "🐧 NixOS Configurations".to_string(),
            command: "nixos-rebuild switch --flake .#<name>".to_string(),
        }
        .print();

        Ok(())
    }
}
