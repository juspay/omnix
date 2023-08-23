use std::collections::BTreeMap;

use leptos::*;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;

use super::{
    show::{FlakeShowOutput, FlakeShowOutputSet, Leaf},
    System,
};

#[serde_as]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PerSystemOutputs(
    // TODO: should not be pub
    #[serde_as(as = "BTreeMap<serde_with::json::JsonString, _>")] pub BTreeMap<System, SystemOutput>,
);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SystemOutput {
    system: System,
    packages: BTreeMap<String, Leaf>,
    legacy_packages: BTreeMap<String, Leaf>,
    devshells: BTreeMap<String, Leaf>,
    checks: BTreeMap<String, Leaf>,
    apps: BTreeMap<String, Leaf>,
    formatter: Option<Leaf>,
    // TODO: Add arbitrary outputs
}

impl SystemOutput {
    pub fn from(output: &FlakeShowOutput, system: &System) -> Self {
        let lookup_type = move |k: &str| -> BTreeMap<String, Leaf> {
            match output.lookup_attrset(vec![k, system.as_ref()]) {
                None => BTreeMap::new(),
                Some(packages) => packages
                    .iter()
                    .filter_map(|(k, v)| {
                        let v = v.as_leaf()?;
                        Some((k.clone(), v.clone()))
                    })
                    .collect(),
            }
        };
        let lookup_leaf_type = move |k: &str| -> Option<Leaf> {
            output.lookup_leaf(vec![k, system.as_ref()]).cloned()
        };
        SystemOutput {
            system: system.clone(),
            packages: lookup_type("packages"),
            legacy_packages: lookup_type("legacyPackages"),
            devshells: lookup_type("devShells"),
            checks: lookup_type("checks"),
            apps: lookup_type("apps"),
            formatter: lookup_leaf_type("formatter"),
        }
    }
}

impl IntoView for SystemOutput {
    fn into_view(self, cx: Scope) -> View {
        // HACK: reconstruct flake show output but just for perSystem, and use
        // its into_view, until we have proper rendering.
        let mut m = BTreeMap::new();
        for (k, v) in [
            ("packages", self.packages),
            // TODO: paginate. rendering nixpkgs legacyPackages takes half a minute!
            ("legacy_packages", self.legacy_packages),
            ("devshells", self.devshells),
            ("checks", self.checks),
            ("apps", self.apps),
        ] {
            if v.is_empty() {
                continue;
            }
            m.insert(
                k.to_string(),
                FlakeShowOutput::Attrset(FlakeShowOutputSet(
                    v.into_iter()
                        .map(|(k, v)| (k, FlakeShowOutput::Leaf(v)))
                        .collect(),
                )),
            );
        }
        match self.formatter {
            None => {}
            Some(v) => {
                m.insert("formatter".to_string(), FlakeShowOutput::Leaf(v));
            }
        }

        let data = FlakeShowOutputSet(m);
        let system = self.system;
        view! { cx,
            <div>
                <h2 class="mt-2 text-xl font-bold text-primary-600">
                    {system.human_readable()} " "
                </h2>
                <span class="mb-2 font-mono text-xs text-gray-500">
                    "(" {system.to_string()} ")"
                </span>
                <div class="text-left">{data}</div>
            </div>
        }
        .into_view(cx)
    }
}
