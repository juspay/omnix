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
        fn view_section_heading(cx: Scope, title: &'static str) -> impl IntoView {
            view! { cx,
                <h3 class="p-2 mt-4 mb-2 font-bold bg-gray-300 border-b-2 border-l-2 border-black text-l">{title}</h3>
            }
        }
        fn view_btree(
            cx: Scope,
            title: &'static str,
            tree: &BTreeMap<String, Leaf>,
        ) -> impl IntoView {
            (!tree.is_empty()).then(|| {
                view! { cx,
                    {view_section_heading(cx, title)}
                    {view_btree_body(cx, tree)}
                }
            })
        }
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
                    {view_btree(cx, "Packages", &self.packages)}
                    {view_btree(cx, "Legacy Packages", &self.legacy_packages)}
                    {view_btree(cx, "Dev Shells", &self.devshells)}
                    {view_btree(cx, "Checks", &self.checks)} {view_btree(cx, "Apps", &self.apps)}
                    {view_section_heading(cx, "Formatter")}
                    {self.formatter.map(|v| {
                        let default = "formatter".to_string();
                        let k = v.name.as_ref().unwrap_or(&default);
                        view_leaf(cx, k, &v)
                    })}
                    {view_section_heading(cx, "Other")}
                    {self.other}
                </div>
            </div>
        }
        .into_view(cx)
    }
}

fn view_btree_body(cx: Scope, tree: &BTreeMap<String, Leaf>) -> View {
    view! { cx,
        <div class="flex flex-wrap justify-start">
            {tree
                .iter()
                .map(|(k, v)| view_leaf(cx, k, v))
                .collect_view(cx)}
        </div>
    }
    .into_view(cx)
}

fn view_leaf(cx: Scope, k: &String, v: &Leaf) -> impl IntoView {
    view! { cx,
        <div
            title=format!("{:?}", v.type_)
            class="flex flex-col p-2 my-2 mr-2 space-y-2 bg-white border-4 border-gray-300 rounded hover:border-gray-400"
        >
            <div class="flex flex-row justify-start space-x-2 font-bold text-primary-500">
                <div>{v.type_.to_icon()}</div>
                <div>{k}</div>
            </div>
            {v
                .name
                .as_ref()
                .map(|v| {
                    view! { cx,
                        <div class="font-mono text-xs text-gray-500">{v}</div>
                    }
                })}

            {v
                .description
                .as_ref()
                .map(|v| {
                    view! { cx, <div class="font-light">{v}</div> }
                })}

        </div>
    }
}
