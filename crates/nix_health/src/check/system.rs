use bytesize::ByteSize;
use serde::{Deserialize, Serialize};

use crate::traits::{Check, CheckResult, Checkable};

/// Check if the system has enough resources
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct System {
    enable: bool,
    required: bool,
    /// Minimum required RAM memory
    min_ram: Option<ByteSize>,
    /// Minimum required disk space
    min_disk_space: Option<ByteSize>,
}

impl Default for System {
    fn default() -> Self {
        Self {
            enable: true,
            required: false,
            // RAM requirements vary between projects.
            min_ram: None,
            // 1TiB is recommended for nix
            min_disk_space: Some(ByteSize::gb(1024)),
        }
    }
}

impl Checkable for System {
    fn check(
        &self,
        nix_info: &nix_rs::info::NixInfo,
        _: Option<nix_rs::flake::url::FlakeUrl>,
    ) -> Vec<Check> {
        let mut checks = vec![];
        let nix_env = &nix_info.nix_env;
        if self.enable {
            if let Some(min_ram) = self.min_ram {
                checks.push(Check {
                    title: "RAM".to_string(),
                    info: format!(
                        "min ram = {:?}; total = {:?}",
                        min_ram, nix_env.total_memory
                    ),
                    result: if nix_env.total_memory < min_ram {
                        CheckResult::Red {
                            msg: format!("Total memory is less than {}", min_ram),
                            suggestion: "Please add more memory to your system".to_string(),
                        }
                    } else {
                        CheckResult::Green
                    },
                    required: self.required,
                });
            };
            if let Some(min_disk_space) = self.min_disk_space {
                checks.push(Check {
                    title: "Disk Space".to_string(),
                    info: format!(
                        "min disk space = {:?}; total = {:?}",
                        min_disk_space, nix_env.total_disk_space
                    ),
                    result: if nix_env.total_disk_space < min_disk_space {
                        CheckResult::Red {
                            msg: format!("Total disk space is less than {}", min_disk_space),
                            suggestion:
                                "The Nix store tends to use a lot of disk space. Please add more disk space"
                                    .to_string(),
                        }
                    } else {
                        CheckResult::Green
                    },
                    required: self.required,
                });
            };
        };
        checks
    }
}
