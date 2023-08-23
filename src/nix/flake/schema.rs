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
    other: Option<FlakeShowOutputSet>,
}

impl FlakeSchema {
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
        let other = output.without_keys(&[
            "packages",
            "legacyPackages",
            "devShells",
            "checks",
            "apps",
            "formatter",
        ]);
        FlakeSchema {
            system: system.clone(),
            packages: lookup_type("packages"),
            legacy_packages: lookup_type("legacyPackages"),
            devshells: lookup_type("devShells"),
            checks: lookup_type("checks"),
            apps: lookup_type("apps"),
            formatter: lookup_leaf_type("formatter"),
            other,
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
                    <div class="text-xl font-bold text-primary-600">
                        {system.human_readable()}
                    </div>
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
