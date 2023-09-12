//! Information about the user's system
use serde::{Deserialize, Serialize};
#[cfg(feature = "all")]
use std::env;
use os_info;

/// Information about the user's system
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SysInfo {
    /// value of $USER
    pub current_user: String,
    pub os: os_info::Type
}

impl SysInfo {
    /// Determine [SysInfo] on the user's system
    #[cfg(feature = "all")]
    pub async fn get_info() -> Result<SysInfo, env::VarError> {
        let current_user = env::var("USER")?;
        let os = os_info::get().os_type();
        Ok(SysInfo {
            current_user,
            os,
        })
    }
}
