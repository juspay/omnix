use serde::Deserialize;

// TODO(idea): What if we provide `om health` like checkmark for each item. Automatically check if the user is in Nix shell or direnv, and ✅ the title accordingly. If not, nudge them to do it.
const OM_SHELL: &str = r#"## Enter the Nix shell

We recommend that you setup nix-direnv (preferably using the convenient template provided at <https://github.com/juspay/nixos-unified-template>). Then run the following in terminal to activate the Nix shell:

```sh-session
direnv allow
```

From this point, anytime you `cd` to this project directory, the Nix shell will be automatically activated.
"#;

const OM_IDE: &str = r#"## IDE or editor setup

>[!IMPORTANT] ❗Make sure you have setup `direnv` as stated above.

You can now launch your favourite editor or IDE from inside the Nix devshell. For VSCode in particular, consult <https://nixos.asia/en/vscode>.

"#;

const DEFAULT: &str = r#"🍾 Welcome to the project

OM_SHELL

---

*(Want to add more instructions here? Add them to the `om.hack.default.readme` field in your `flake.nix` file)*
"#;

/// The README to display at the end.
///
/// Placeholder parameters:
/// - `OM_SHELL`: Instructions to enter the Nix shell.
/// - `OM_IDE`: Instructions to setup the IDE.
#[derive(Debug, Deserialize, Clone)]
pub struct Readme(pub String);

impl Default for Readme {
    fn default() -> Self {
        Self(DEFAULT.to_string())
    }
}

impl Readme {
    /// Get the Markdown string, after doing parameter replacements.
    pub fn get_markdown(&self) -> String {
        self.0
            .replace("OM_SHELL", OM_SHELL)
            .replace("OM_IDE", OM_IDE)
    }
}
