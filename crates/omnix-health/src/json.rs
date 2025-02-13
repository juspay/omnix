//! JSON output schema for health checks
use crate::traits::Check;
use anyhow::Context;
use bytesize::ByteSize;
use nix_rs::{
    env::{NixInstaller, OS},
    flake::system::System,
    info::NixInfo,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Entire JSON health check output
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HealthOutput {
    /// Map of check names to their results
    pub checks: HashMap<String, Check>,
    /// System environment information
    pub info: HealthEnvInfo,
}

impl HealthOutput {
    pub async fn get(checks: Vec<(&'static str, Check)>) -> anyhow::Result<Self> {
        Ok(Self {
            checks: checks
                .into_iter()
                .map(|(k, v)| (k.to_string(), v))
                .collect(),
            info: HealthEnvInfo::get().await?,
        })
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HealthEnvInfo {
    nix_installer: NixInstaller,
    system: System,
    os: OS,
    total_memory: ByteSize,
    total_disk_space: ByteSize,
}

impl HealthEnvInfo {
    /// Get system environment information
    ///
    /// Returns error if [NixInfo] cannot be retrieved
    pub async fn get() -> anyhow::Result<Self> {
        let nix_info = NixInfo::get()
            .await
            .as_ref()
            .context("Unable to gather nix info")?;

        Ok(Self {
            nix_installer: nix_info.nix_env.installer.clone().into(),
            system: nix_info.nix_config.system.value.clone(),
            os: nix_info.nix_env.os.clone(),
            total_memory: nix_info.nix_env.total_memory,
            total_disk_space: nix_info.nix_env.total_disk_space,
        })
    }
}
