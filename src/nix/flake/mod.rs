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
    pub url: FlakeUrl,
    /// `nix flake show` output
    pub output: FlakeOutput,
    // TODO: Add higher-level info
    #[serde_as(as = "BTreeMap<serde_with::json::JsonString, _>")]
    pub per_system: BTreeMap<System, SystemOutput>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SystemOutput {
    packages: BTreeMap<String, Leaf>,
    devshells: BTreeMap<String, Leaf>,
    checks: BTreeMap<String, Leaf>,
    apps: BTreeMap<String, Leaf>,
}

impl SystemOutput {
    pub fn from(output: &FlakeOutput, system: System) -> Self {
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

/// Get [Flake] info for the given flake url
#[instrument(name = "flake")]
#[server(GetFlake, "/api")]
pub async fn get_flake(url: FlakeUrl) -> Result<Flake, ServerFnError> {
    let output = self::show::run_nix_flake_show(&url).await?;
    let mut per_system = BTreeMap::new();
    for system in [
        "x86_64-linux",
        "x86_64-darwin",
        "aarch64-linux",
        "aarch64-darwin",
    ] {
        per_system.insert(
            System::from(system),
            SystemOutput::from(&output, System::from(system)),
        );
    }
    Ok(Flake {
        url,
        output,
        per_system,
    })
}

impl IntoView for Flake {
    // TODO: Remove this attribute
    #[allow(clippy::iter_kv_map)]
    fn into_view(self, cx: Scope) -> View {
        view! { cx,
            <div class="flex flex-col my-4">
                <h3 class="text-lg font-bold">{self.url}</h3>
                <p class="my-2">
                    TODO: Show overview, rather than raw flake output
                    {self
                        .per_system
                        .iter()
                        .map(|(k, _v)| {
                            let system = &k.to_string();

                            view! { cx,
                                <li>
                                    <a href=format!("/flake/{}", system)>{system}</a>
                                </li>
                            }
                        })
                        .collect_view(cx)}
                </p>
                <div class="font-mono text-sm">{self.output}</div>
            </div>
        }
        .into_view(cx)
    }
}

impl IntoView for SystemOutput {
    fn into_view(self, cx: Scope) -> View {
        view! { cx, <pre>"TODO: Per System"</pre> }
        .into_view(cx)
    }
}
