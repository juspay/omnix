use std::fmt::{Display, Formatter};

use serde::{Deserialize, Serialize};

/// The system for which a derivation will build
///
/// The enum includes the four standard systems, as well as a fallback to
/// capture the rest.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum System {
    Linux(Arch),
    Darwin(Arch),
    Other(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Arch {
    Aarch64,
    X86_64,
}

impl From<&str> for System {
    fn from(s: &str) -> Self {
        match s {
            "aarch64-linux" => Self::Linux(Arch::Aarch64),
            "x86_64-linux" => Self::Linux(Arch::X86_64),
            "x86_64-darwin" => Self::Darwin(Arch::X86_64),
            "aarch64-darwin" => Self::Darwin(Arch::Aarch64),
            _ => Self::Other(s.to_string()),
        }
    }
}

impl From<String> for System {
    fn from(s: String) -> Self {
        Self::from(s.as_str())
    }
}

impl AsRef<str> for System {
    fn as_ref(&self) -> &str {
        match self {
            System::Linux(Arch::Aarch64) => "aarch64-linux",
            System::Linux(Arch::X86_64) => "x86_64-linux",
            System::Darwin(Arch::X86_64) => "x86_64-darwin",
            System::Darwin(Arch::Aarch64) => "aarch64-darwin",
            System::Other(s) => s,
        }
    }
}

impl From<System> for String {
    fn from(s: System) -> Self {
        s.as_ref().to_string()
    }
}

impl Display for System {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_ref())
    }
}

impl System {
    pub fn human_readable(&self) -> String {
        match self {
            System::Linux(arch) => format!("Linux ({})", arch.human_readable()),
            System::Darwin(arch) => format!("macOS ({})", arch.human_readable()),
            System::Other(s) => s.clone(),
        }
    }
}

impl Arch {
    pub fn human_readable(&self) -> &'static str {
        match self {
            Self::Aarch64 => "ARM",
            Self::X86_64 => "Intel",
        }
    }
}
