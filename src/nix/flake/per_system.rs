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
    packages: BTreeMap<String, Leaf>,
    devshells: BTreeMap<String, Leaf>,
    checks: BTreeMap<String, Leaf>,
    apps: BTreeMap<String, Leaf>,
}

impl SystemOutput {
    pub fn from(output: &FlakeShowOutput, system: System) -> Self {
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
        SystemOutput {
            packages: lookup_type("packages"),
            devshells: lookup_type("devShells"),
            checks: lookup_type("checks"),
            apps: lookup_type("apps"),
        }
    }
}

impl IntoView for SystemOutput {
    fn into_view(self, cx: Scope) -> View {
        let mut m = BTreeMap::new();
        for k in [
            ("packages", self.packages),
            ("devshells", self.devshells),
            ("checks", self.checks),
            ("apps", self.apps),
        ] {
            m.insert(
                k.0.to_string(),
                FlakeShowOutput::Attrset(FlakeShowOutputSet(
                    k.1.into_iter()
                        .map(|(k, v)| (k, FlakeShowOutput::Leaf(v)))
                        .collect(),
                )),
            );
        }

        let data = FlakeShowOutputSet(m);
        data.into_view(cx)
    }
}
