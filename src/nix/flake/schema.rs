use std::collections::BTreeMap;

use leptos::*;
use serde::{Deserialize, Serialize};

use super::{
    show::{FlakeShowOutput, FlakeShowOutputSet, Leaf},
    System,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FlakeSchema {
    system: System,
    packages: BTreeMap<String, Leaf>,
    legacy_packages: BTreeMap<String, Leaf>,
    devshells: BTreeMap<String, Leaf>,
    checks: BTreeMap<String, Leaf>,
    apps: BTreeMap<String, Leaf>,
    formatter: Option<Leaf>,
    /// Other unrecognized keys.
    other: Option<FlakeShowOutputSet>,
    // TODO: Add nixosModules, nixosConfigurations, darwinModules, etc.
}

impl FlakeSchema {
    /// Builds the [FlakeSchema] for the given system
    ///
    /// Other system outputs are eliminated, but non-per-system outputs are kept
    /// as is (in [FlakeSchema::other]).
    pub fn from(output: &FlakeShowOutput, system: &System) -> Self {
        let output: &mut FlakeShowOutput = &mut output.clone();
        let lookup_type = |output: &mut FlakeShowOutput, k: &str| -> BTreeMap<String, Leaf> {
            let r = match output.pop(vec![k, system.as_ref()]) {
                None => BTreeMap::new(),
                Some(out) => match out.as_attrset() {
                    None => BTreeMap::new(),
                    Some(packages) => packages
                        .iter()
                        .filter_map(|(k, v)| {
                            let v = v.as_leaf()?;
                            Some((k.clone(), v.clone()))
                        })
                        .collect(),
                },
            };
            output.pop(vec![k]);
            r
        };
        let lookup_leaf_type = |output: &mut FlakeShowOutput, k: &str| -> Option<Leaf> {
            let leaf = output.pop(vec![k, system.as_ref()])?.as_leaf()?.clone();
            output.pop(vec![k]);
            Some(leaf)
        };
        FlakeSchema {
            system: system.clone(),
            packages: lookup_type(output, "packages"),
            legacy_packages: lookup_type(output, "legacyPackages"),
            devshells: lookup_type(output, "devShells"),
            checks: lookup_type(output, "checks"),
            apps: lookup_type(output, "apps"),
            formatter: lookup_leaf_type(output, "formatter"),
            other: (*output)
                .as_attrset()
                .map(|v| FlakeShowOutputSet(v.clone())),
        }
    }

    // HACK: reconstruct flake show output but just for perSystem, and use
    // its into_view, until we have proper rendering.
    pub fn to_flake_show_output(self) -> FlakeShowOutputSet {
        let mut m = BTreeMap::new();
        for (k, v) in [
            ("packages", self.packages.clone()),
            ("legacyPackages", self.legacy_packages.clone()),
            ("devShells", self.devshells.clone()),
            ("checks", self.checks.clone()),
            ("apps", self.apps.clone()),
        ] {
            if v.is_empty() {
                continue;
            }
            let inner = FlakeShowOutput::Attrset(FlakeShowOutputSet(
                v.into_iter()
                    .map(|(k, v)| (k, FlakeShowOutput::Leaf(v)))
                    .collect(),
            ));
            let outer = FlakeShowOutput::Attrset(FlakeShowOutputSet(
                vec![(self.system.to_string(), inner)].into_iter().collect(),
            ));
            m.insert(k.to_string(), outer);
        }
        match self.formatter {
            None => {}
            Some(v) => {
                m.insert("formatter".to_string(), FlakeShowOutput::Leaf(v));
            }
        }
        match self.other {
            None => {}
            Some(v) => {
                m.extend(v.0);
            }
        }

        FlakeShowOutputSet(m)
    }
}

impl IntoView for FlakeSchema {
    fn into_view(self, cx: Scope) -> View {
        let system = self.system.clone();
        let data = self.to_flake_show_output();
        view! { cx,
            <div>
                <h2 class="my-2 ">
                    <div class="text-xl font-bold text-primary-600">{system.human_readable()}</div>
                    " "
                    <span class="font-mono text-xs text-gray-500">
                        "(" {system.to_string()} ")"
                    </span>
                </h2>
                <div class="text-left">{data}</div>
            </div>
        }
        .into_view(cx)
    }
}
