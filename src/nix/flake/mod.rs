pub mod show;
pub mod system;
pub mod url;

use serde_with::serde_as;
use std::collections::BTreeMap;

use leptos::*;
use serde::{Deserialize, Serialize};
use tracing::instrument;

use self::{
    show::{FlakeOutput, Leaf},
    system::System,
    url::FlakeUrl,
};

/// All the information about a Nix flake
#[serde_as]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Flake {
    /// The flake url which this struct represents
    url: FlakeUrl,
    /// `nix flake show` output
    output: FlakeOutput,
    // TODO: Add higher-level info
    #[serde_as(as = "BTreeMap<serde_with::json::JsonString, _>")]
    per_system: BTreeMap<System, SystemOutput>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SystemOutput {
    packages: BTreeMap<String, Leaf>,
    dev_shells: BTreeMap<String, Leaf>,
    checks: BTreeMap<String, Leaf>,
    apps: BTreeMap<String, Leaf>,
}

impl SystemOutput {
    pub fn from(output: &FlakeOutput, system: System) -> Self {
        let lookup_type = move |k: &str| -> BTreeMap<String, Leaf> {
            match output.lookup_attrset(vec!["packages", system.as_ref()]) {
                None => BTreeMap::new(),
                Some(packages) => {
                    let packages: BTreeMap<String, Leaf> = packages
                        .iter()
                        .filter_map(|(k, v)| {
                            let v = v.as_leaf()?;
                            Some((k.clone(), v.clone()))
                        })
                        .collect();
                    packages
                }
            }
        };
        SystemOutput {
            packages: lookup_type("packages"),
            dev_shells: lookup_type("devShells"),
            checks: lookup_type("checks"),
            apps: lookup_type("apps"),
        }
    }
}

/// Get [Flake] info for the given flake url
#[instrument(name = "flake")]
#[server(GetFlake, "/api")]
pub async fn get_flake(url: FlakeUrl) -> Result<Flake, ServerFnError> {
    let output = self::show::run_nix_flake_show(&url).await?;
    let mut per_system = BTreeMap::new();
    per_system.insert(
        System::from("aarch64-darwin"),
        SystemOutput::from(&output, System::from("aarch64-darwin")),
    );
    Ok(Flake {
        url,
        output,
        per_system,
    })
}

impl IntoView for Flake {
    fn into_view(self, cx: Scope) -> View {
        view! { cx,
            <div class="flex flex-col my-4">
                <h3 class="text-lg font-bold">{self.url}</h3>
                <p class="my-2">
                    TODO: Show overview, rather than raw flake output
                </p>
                <div class="font-mono text-sm">{self.output}</div>
            </div>
        }
        .into_view(cx)
    }
}
