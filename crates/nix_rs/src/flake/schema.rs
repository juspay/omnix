//! High-level schema of a flake
//!
//! TODO: Consolidate with `outputs.rs`
#![allow(missing_docs)]
use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use super::{
    outputs::{FlakeOutputs, InventoryItem, Leaf},
    System,
};

/// High-level schema of a flake
///
/// TODO: Consolidate with `outputs.rs`
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FlakeSchema {
    pub system: System,
    pub packages: BTreeMap<String, Leaf>,
    pub legacy_packages: BTreeMap<String, Leaf>,
    pub devshells: BTreeMap<String, Leaf>,
    pub checks: BTreeMap<String, Leaf>,
    pub apps: BTreeMap<String, Leaf>,
    pub formatter: Option<Leaf>,
    pub nixos_configurations: BTreeMap<String, Leaf>,
    pub darwin_configurations: BTreeMap<String, Leaf>,
    pub home_configurations: BTreeMap<String, Leaf>,
    pub nixos_modules: BTreeMap<String, Leaf>,
    pub docker_images: BTreeMap<String, Leaf>,
    pub overlays: BTreeMap<String, Leaf>,
    pub templates: BTreeMap<String, Leaf>,
    pub schemas: BTreeMap<String, Leaf>,
    /// Other unrecognized keys.
    pub other: Option<FlakeOutputs>,
}

impl FlakeSchema {
    /// Builds the [FlakeSchema] for the given system
    ///
    /// Other system outputs are eliminated, but non-per-system outputs are kept
    /// as is (in [FlakeSchema::other]).
    pub fn from(output: &FlakeOutputs, system: &System) -> Self {
        let output: &mut FlakeOutputs = &mut output.clone();
        let pop_tree = |inventory_item: &mut Option<&mut InventoryItem>,
                        ks: &[&str]|
         -> BTreeMap<String, Leaf> {
            let mut result = BTreeMap::new();

            if let Some(item) = inventory_item {
                if let Some(out) = item.pop(ks) {
                    if let Some(outs) = out.as_attrset() {
                        for (k, v) in outs {
                            if let Some(leaf) = v.as_leaf() {
                                result.insert(k.clone(), leaf.clone());
                            }
                        }
                    }
                }
                item.pop(ks);
            }

            result
        };
        let pop_per_system_tree = |output: &mut FlakeOutputs, k: &str| -> BTreeMap<String, Leaf> {
            pop_tree(
                &mut output.inventory.get_mut(k),
                &["children", system.as_ref(), "children"],
            )
        };
        let pop_leaf_type = |output: &mut FlakeOutputs, k: &str| -> Option<Leaf> {
            let inventory_item = output.inventory.get_mut(k)?;
            let leaf = inventory_item
                .pop(&["children", system.as_ref()])?
                .as_leaf()?
                .clone();
            inventory_item.pop(&[k]);
            Some(leaf)
        };

        FlakeSchema {
            system: system.clone(),
            packages: pop_per_system_tree(output, "packages"),
            legacy_packages: pop_per_system_tree(output, "legacyPackages"),
            devshells: pop_per_system_tree(output, "devShells"),
            checks: pop_per_system_tree(output, "checks"),
            apps: pop_per_system_tree(output, "apps"),
            formatter: pop_leaf_type(output, "formatter"),
            nixos_configurations: pop_tree(
                &mut output.inventory.get_mut("nixosConfigurations"),
                &["children"],
            ),
            darwin_configurations: pop_tree(
                &mut output.inventory.get_mut("darwinConfigurations"),
                &["children"],
            ),
            home_configurations: pop_tree(
                &mut output.inventory.get_mut("homeConfigurations"),
                &["children"],
            ),
            nixos_modules: pop_tree(&mut output.inventory.get_mut("nixosModules"), &["children"]),
            docker_images: pop_tree(&mut output.inventory.get_mut("dockerImages"), &["children"]),
            overlays: pop_tree(&mut output.inventory.get_mut("overlays"), &["children"]),
            templates: pop_tree(&mut output.inventory.get_mut("templates"), &["children"]),
            schemas: pop_tree(&mut output.inventory.get_mut("schemas"), &["children"]),
            other: Some(output.clone()),
        }
    }
}
