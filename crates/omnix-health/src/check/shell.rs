use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use crate::traits::{Check, CheckResult, Checkable};

#[derive(Debug, Serialize, Deserialize, Clone, Hash, Eq, PartialEq)]
pub struct ShellCheck {
    pub(crate) enable: bool,
    /// Whether to produce [Check::required] checks
    pub(crate) required: bool,
}

impl Default for ShellCheck {
    fn default() -> Self {
        Self {
            enable: true,
            required: false,
        }
    }
}

impl Checkable for ShellCheck {
    fn check(
        &self,
        _nix_info: &nix_rs::info::NixInfo,
        _flake: Option<&nix_rs::flake::url::FlakeUrl>,
    ) -> Vec<Check> {
        if !self.enable {
            return vec![];
        }
        let user_shell_env = match CurrentUserShellEnv::new() {
            Ok(shell) => shell,
            Err(err) => {
                tracing::error!("Skipping shell dotfile check! {:?}", err);
                if self.required {
                    panic!("Unable to determine user's shell environment (see above)");
                } else {
                    tracing::warn!("Skipping shell dotfile check! (see above)");
                    return vec![];
                }
            }
        };

        // Iterate over each dotfile and check if it is managed by Nix
        let mut managed: HashMap<&'static str, PathBuf> = HashMap::new();
        let mut unmanaged: HashMap<&'static str, PathBuf> = HashMap::new();
        for (name, path) in user_shell_env.dotfiles {
            if super::direnv::is_path_in_nix_store(&path) {
                managed.insert(name, path.clone());
            } else {
                unmanaged.insert(name, path.clone());
            }
        }

        let title = "Shell dotfiles".to_string();
        let info = format!(
            "Shell={:?}; HOME={:?}; Managed: {:?}; Unmanaged: {:?}",
            user_shell_env.shell, user_shell_env.home, managed, unmanaged
        );
        let result = if !managed.is_empty() && unmanaged.is_empty() {
            // If *all* dotfiles are managed, then we are good
            CheckResult::Green
        } else {
            CheckResult::Red {
                msg: format!("Default Shell: {:?} is not managed by Nix", user_shell_env.shell),
                    suggestion: "You can use `home-manager` to manage shell configuration. See <https://github.com/juspay/nixos-unified-template>".to_string(),
            }
        };
        let check = Check {
            title,
            info,
            result,
            required: self.required,
        };

        vec![check]
    }
}

/// The shell environment of the current user
struct CurrentUserShellEnv {
    /// The user's home directory
    home: PathBuf,
    /// Current shell
    shell: Shell,
    /// *Absolute* paths to the dotfiles
    dotfiles: HashMap<&'static str, PathBuf>,
}

impl CurrentUserShellEnv {
    /// Get the current user's shell environment
    fn new() -> Result<Self, ShellError> {
        let home = PathBuf::from(std::env::var("HOME")?);
        let shell = Shell::current_shell()?;
        let dotfiles = shell.get_dotfiles(&home)?;
        let v = CurrentUserShellEnv {
            home,
            shell,
            dotfiles,
        };
        Ok(v)
    }
}

#[derive(thiserror::Error, Debug)]
enum ShellError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Environment variable error: {0}")]
    Var(#[from] std::env::VarError),

    #[error("Bad $SHELL value")]
    BadShellPath,

    #[error("Unsupported shell. Please file an issue at <https://github.com/juspay/omnix/issues>")]
    UnsupportedShell,
}

/// An Unix shell
#[derive(Debug, Serialize, Deserialize, Clone, Hash, Eq, PartialEq)]
#[serde(rename_all = "lowercase")]
enum Shell {
    Zsh,
    Bash,
}

impl Shell {
    /// Returns the user's current [Shell]
    fn current_shell() -> Result<Self, ShellError> {
        let shell_path = PathBuf::from(std::env::var("SHELL")?);
        Self::from_path(shell_path)
    }

    /// Lookup [Shell] from the given executable path
    /// For example if path is `/bin/zsh`, it would return `Zsh`
    fn from_path(exe_path: PathBuf) -> Result<Self, ShellError> {
        let shell_name = exe_path
            .file_name()
            .ok_or(ShellError::BadShellPath)?
            .to_string_lossy();

        match shell_name.as_ref() {
            "zsh" => Ok(Shell::Zsh),
            "bash" => Ok(Shell::Bash),
            _ => Err(ShellError::UnsupportedShell),
        }
    }

    /// Get shell dotfiles
    fn dotfile_names(&self) -> Vec<&'static str> {
        match &self {
            Shell::Zsh => vec![".zshrc", ".zshenv", ".zprofile"],
            Shell::Bash => vec![".bashrc", ".bash_profile", ".profile"],
        }
    }

    /// Get the currently existing dotfiles under $HOME
    ///
    /// Returned paths will be absolute (i.e., symlinks are resolved).
    fn get_dotfiles(&self, home_dir: &Path) -> std::io::Result<HashMap<&'static str, PathBuf>> {
        let mut paths = HashMap::new();
        for dotfile in self.dotfile_names() {
            match std::fs::canonicalize(home_dir.join(dotfile)) {
                Ok(path) => {
                    paths.insert(dotfile, path);
                }
                Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
                    // If file not found, skip
                }
                Err(err) => return Err(err),
            }
        }
        Ok(paths)
    }
}
