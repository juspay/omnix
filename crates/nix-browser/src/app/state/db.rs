//! A database of [Flake] intended to be cached in dioxus [Signal] and persisted to disk.
//!
//! This is purposefully dumb right now, but we might revisit this in future based on actual performance.

use serde::{Deserialize, Serialize};
use std::{collections::HashMap, time::SystemTime};

use dioxus_sdk::storage::new_storage;
use dioxus_sdk::storage::LocalStorage;
use dioxus_signals::Signal;

use crate::app::state::FlakeUrl;
use nix_rs::flake::Flake;

/// A database of [Flake] intended to be cached in dioxus [Signal] and persisted to disk.
///
/// Contains the "last fetched" time and the [Flake] itself.
#[derive(Default, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FlakeCache(HashMap<FlakeUrl, Option<(SystemTime, Flake)>>);

impl FlakeCache {
    /// Create a new [Signal] for [FlakeCache] from [LocalStorage].
    pub fn new_signal() -> Signal<FlakeCache> {
        new_storage::<LocalStorage, _>("flake_cache".to_string(), || {
            tracing::warn!("📦 No flake cache found");
            let init = FlakeUrl::suggestions()
                .into_iter()
                .map(|url| (url, None))
                .collect();
            FlakeCache(init)
        })
    }

    /// Look up a [Flake] by [FlakeUrl] in the cache.
    pub fn get(&self, k: &FlakeUrl) -> Option<Flake> {
        let (t, flake) = self.0.get(k).and_then(|v| v.as_ref().cloned())?;
        tracing::info!("Cache hit for {} (updated: {:?})", k, t);
        Some(flake)
    }

    /// Update the cache with a new [Flake].
    pub fn update(&mut self, k: FlakeUrl, flake: Flake) {
        tracing::info!("Caching flake [{}]", &k);
        self.0.insert(k, Some((SystemTime::now(), flake)));
    }

    /// Recently updated flakes, along with any unavailable flakes in cache.
    pub fn recent_flakes(&self) -> Vec<FlakeUrl> {
        let mut pairs: Vec<_> = self
            .0
            .iter()
            .map(|(k, v)| (k, v.as_ref().map(|(t, _)| t)))
            .collect();

        // Sort by the timestamp in descending order.
        pairs.sort_unstable_by(|a, b| b.1.cmp(&a.1));

        pairs.into_iter().map(|(k, _)| k.clone()).collect()
    }
}
