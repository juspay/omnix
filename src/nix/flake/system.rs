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
        /*
        match s {
            System::Linux(Arch::Aarch64) => "aarch64-linux".to_string(),
            System::Linux(Arch::X86_64) => "x86_64-linux".to_string(),
            System::Darwin(Arch::X86_64) => "x86_64-darwin".to_string(),
            System::Darwin(Arch::Aarch64) => "aarch64-darwin".to_string(),
            System::Other(s) => s,
        }
        */
    }
}
