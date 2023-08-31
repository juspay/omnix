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
    /// Other unrecognized keys.
    pub other: Option<BTreeMap<String, FlakeOutputs>>,
    // TODO: Add nixosModules, nixosConfigurations, darwinModules, etc.
}

impl FlakeSchema {
    /// Builds the [FlakeSchema] for the given system
    ///
    /// Other system outputs are eliminated, but non-per-system outputs are kept
    /// as is (in [FlakeSchema::other]).
    pub fn from(output: &FlakeOutputs, system: &System) -> Self {
        let output: &mut FlakeOutputs = &mut output.clone();
        let pop_per_system_tree = |output: &mut FlakeOutputs, k: &str| -> BTreeMap<String, Val> {
            let mut f = || -> Option<BTreeMap<String, Val>> {
                let out = output.pop(&[k, system.as_ref()])?;
                let outs = out.as_attrset()?;
                let r = outs
                    .iter()
                    .filter_map(|(k, v)| {
                        let v = v.as_leaf()?;
                        Some((k.clone(), v.clone()))
                    })
                    .collect();
                Some(r)
            };
            let mr = f();
            output.pop(&[k]);
            mr.unwrap_or(BTreeMap::new())
        };
        let pop_leaf_type = |output: &mut FlakeOutputs, k: &str| -> Option<Val> {
            let leaf = output.pop(&[k, system.as_ref()])?.as_leaf()?.clone();
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
            other: (*output).as_attrset().cloned(),
        }
    }
}
