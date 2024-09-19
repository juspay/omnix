use std::io::IsTerminal;

use anyhow::Context;
use clap::Parser;
use colored::Colorize;
use nix_rs::{
    command::NixCmd,
    config::NixConfig,
    flake::{outputs::InventoryItem, url::FlakeUrl, Flake},
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
    /// Convert a [InventoryItem] to vector of [Row]s
    pub fn from_inventory_for(path: &[&str], output: Option<&InventoryItem>) -> Vec<Row> {
        match output {
            Some(out) => out
                .lookup_returning_qualified_attributes(path)
                .map(|v| {
                    v.iter()
                        .map(|(name, leaf)| Row {
                            name: name.to_owned(),
                            description: leaf
                                .as_val()
                                .and_then(|val| val.short_description.as_deref())
                                .filter(|s| !s.is_empty())
                                .unwrap_or(&String::from("N/A"))
                                .to_owned(),
                        })
                        .collect()
                })
                .unwrap_or_default(),
            None => vec![],
        }
    }
}

impl ShowCommand {
    pub async fn run(&self) -> anyhow::Result<()> {
        let nix_cmd = NixCmd::get().await;
        let nix_config = NixConfig::get().await.as_ref()?;
        let system = &nix_config.system.value;
        let flake = Flake::from_nix(nix_cmd, nix_config, self.flake_url.clone())
            .await
            .with_context(|| "Unable to fetch flake")?;

        let print_flake_output_table = |title: &str, output: &str, command: Option<String>| {
            FlakeOutputTable {
                rows: Row::from_inventory_for(&[], flake.output.get_inventory(output.to_string())),
                title: title.to_string(),
                command,
            }
            .print();
        };

        let print_per_system_flake_output_table =
            |title: &str, output: &str, command: Option<String>| {
                FlakeOutputTable {
                    rows: Row::from_inventory_for(
                        &[system.as_ref()],
                        flake.output.get_inventory(output.to_string()),
                    ),
                    title: title.to_string(),
                    command,
                }
                .print();
            };

        print_per_system_flake_output_table(
            "📦 Packages",
            "packages",
            Some(format!("nix build {}#<name>", self.flake_url)),
        );
        print_per_system_flake_output_table(
            "🐚 Devshells",
            "devShells",
            Some(format!("nix develop {}#<name>", self.flake_url)),
        );
        print_per_system_flake_output_table(
            "🚀 Apps",
            "apps",
            Some(format!("nix run {}#<name>", self.flake_url)),
        );
        print_per_system_flake_output_table(
            "🔍 Checks",
            "checks",
            Some("nix flake check".to_string()),
        );

        print_flake_output_table(
            "🐧 NixOS Configurations",
            "nixosConfigurations",
            Some(format!(
                "nixos-rebuild switch --flake {}#<name>",
                self.flake_url
            )),
        );
        print_flake_output_table(
            "🍏 Darwin Configurations",
            "darwinConfigurations",
            Some(format!(
                "darwin-rebuild switch --flake {}#<name>",
                self.flake_url
            )),
        );
        print_flake_output_table("🔧 NixOS Modules", "nixosModules", None);
        print_flake_output_table(
            "🐳 Docker Images",
            "dockerImages",
            Some(format!("nix build {}#dockerImages.<name>", self.flake_url)),
        );
        print_flake_output_table("🎨 Overlays", "overlays", None);
        print_flake_output_table(
            "📝 Templates",
            "templates",
            Some(format!("nix flake init -t {}#<name>", self.flake_url)),
        );
        print_flake_output_table("📜 Schemas", "schemas", None);

        Ok(())
    }
}
