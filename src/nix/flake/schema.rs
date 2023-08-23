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
        let pop_type = |output: &mut FlakeShowOutput, k: &str| -> BTreeMap<String, Leaf> {
            let mut f = || -> Option<BTreeMap<String, Leaf>> {
                let out = output.pop(vec![k, system.as_ref()])?;
                let packages = out.as_attrset()?;
                let r = packages
                    .0
                    .iter()
                    .filter_map(|(k, v)| {
                        let v = v.as_leaf()?;
                        Some((k.clone(), v.clone()))
                    })
                    .collect();
                Some(r)
            };
            let mr = f();
            output.pop(vec![k]);
            mr.unwrap_or(BTreeMap::new())
        };
        let pop_leaf_type = |output: &mut FlakeShowOutput, k: &str| -> Option<Leaf> {
            let leaf = output.pop(vec![k, system.as_ref()])?.as_leaf()?.clone();
            output.pop(vec![k]);
            Some(leaf)
        };
        FlakeSchema {
            system: system.clone(),
            packages: pop_type(output, "packages"),
            legacy_packages: pop_type(output, "legacyPackages"),
            devshells: pop_type(output, "devShells"),
            checks: pop_type(output, "checks"),
            apps: pop_type(output, "apps"),
            formatter: pop_leaf_type(output, "formatter"),
            other: (*output).as_attrset().cloned(),
        }
    }
}

impl IntoView for FlakeSchema {
    fn into_view(self, cx: Scope) -> View {
        let system = &self.system.clone();
        view! { cx,
            <div>
                <h2 class="my-2 ">
                    <div class="text-xl font-bold text-primary-600">{system.human_readable()}</div>
                    " "
                    <span class="font-mono text-xs text-gray-500">
                        "(" {system.to_string()} ")"
                    </span>
                </h2>

                <div class="text-left">

                    <h3 class="my-2 font-bold text-l">"Packages"</h3>
                    {leaf_map(cx, &self.packages)}
                    <h3 class="my-2 font-bold text-l">"Dev Shells"</h3>
                    {leaf_map(cx, &self.devshells)}
                    <h3 class="my-2 font-bold text-l">"Checks"</h3>
                    {leaf_map(cx, &self.checks)}
                    <h3 class="my-2 font-bold text-l">"Apps"</h3>
                    {leaf_map(cx, &self.apps)}
                    <h3 class="my-2 font-bold text-l">"Legacy Packages"</h3>
                    {leaf_map(cx, &self.legacy_packages)}
                    <h3 class="my-2 font-bold text-l">"Formatter"</h3>
                    {self.formatter}
                    <h3 class="my-2 font-bold text-l">"Other"</h3>
                    {self.other}
                </div>
            </div>
        }
        .into_view(cx)
    }
}

fn leaf_map(cx: Scope, t: &BTreeMap<String, Leaf>) -> View {
    view! { cx,
        <ul class="list-disc">
            {t
                .into_iter()
                .map(|(k, v)| {
                    view! { cx,
                        <li class="ml-4">
                            <span class="px-2 py-1 font-bold text-primary-500">{k}</span>
                            {v.clone()}
                        </li>
                    }
                })
                .collect_view(cx)}
        </ul>
    }
    .into_view(cx)
}
