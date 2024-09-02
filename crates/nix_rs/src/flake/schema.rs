//! High-level schema of a flake
//!
//! TODO: Use <https://github.com/DeterminateSystems/flake-schemas>
use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use super::{
    outputs::{FlakeOutputs, Val},
    System,
};

/// High-level schema of a flake
///
/// TODO: Use <https://github.com/DeterminateSystems/flake-schemas>
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FlakeSchema {
    pub system: System,
    pub packages: BTreeMap<String, Val>,
    pub legacy_packages: BTreeMap<String, Val>,
    pub devshells: BTreeMap<String, Val>,
    pub checks: BTreeMap<String, Val>,
    pub apps: BTreeMap<String, Val>,
    pub formatter: Option<Val>,
    pub nixos_configurations: BTreeMap<String, Val>,
    pub darwin_configurations: BTreeMap<String, Val>,
    pub home_configurations: BTreeMap<String, Val>,
    pub nixos_modules: BTreeMap<String, Val>,
    pub docker_images: BTreeMap<String, Val>,
    pub overlays: BTreeMap<String, Val>,
    pub templates: BTreeMap<String, Val>,
    pub schemas: BTreeMap<String, Val>,
    /// Other unrecognized keys.
    pub other: Option<BTreeMap<String, FlakeOutputs>>,
}

impl FlakeSchema {
    /// Builds the [FlakeSchema] for the given system
    ///
    /// Other system outputs are eliminated, but non-per-system outputs are kept
    /// as is (in [FlakeSchema::other]).
    pub fn from(output: &FlakeOutputs, system: &System) -> Self {
        let output: &mut FlakeOutputs = &mut output.clone();
        let pop_tree = |output: &mut FlakeOutputs, ks: &[&str]| -> BTreeMap<String, Val> {
            let mut f = || -> Option<BTreeMap<String, Val>> {
                let out = output.pop(ks)?;
                let outs = out.as_attrset()?;
                let r = outs
                    .iter()
                    .filter_map(|(k, v)| {
                        let v = v.as_val()?;
                        Some((k.clone(), v.clone()))
                    })
                    .collect();
                Some(r)
            };
            let mr = f();
            output.pop(ks);
            mr.unwrap_or(BTreeMap::new())
        };
        let pop_per_system_tree = |output: &mut FlakeOutputs, k: &str| -> BTreeMap<String, Val> {
            pop_tree(
                output,
                &[k, "output", "children", system.as_ref(), "children"],
            )
        };
        let pop_leaf_type = |output: &mut FlakeOutputs, k: &str| -> Option<Val> {
            let leaf = output
                .pop(&[k, "output", "children", system.as_ref()])?
                .as_val()?
                .clone();
            output.pop(&[k]);
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
            nixos_configurations: pop_tree(output, &["nixosConfigurations", "output", "children"]),
            darwin_configurations: pop_tree(
                output,
                &["darwinConfigurations", "output", "children"],
            ),
            home_configurations: pop_tree(output, &["homeConfigurations", "output", "children"]),
            nixos_modules: pop_tree(output, &["nixosModules", "output", "children"]),
            docker_images: pop_tree(output, &["dockerImages", "output", "children"]),
            overlays: pop_tree(output, &["overlays", "output", "children"]),
            templates: pop_tree(output, &["templates", "output", "children"]),
            schemas: pop_tree(output, &["schemas", "output", "children"]),
            other: (*output).as_attrset().cloned(),
        }
    }
}
