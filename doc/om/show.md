# Show

The `om show` command seeks to provide a better `nix flake show` experience.

> [!WARNING]
> Currently, `om show` is a wrapper on `nix flake show`, but with support for [flake schemas](https://github.com/NixOS/nix/pull/8892). More is planned for `om show`. See [issue #162](https://github.com/juspay/omnix/issues/162).

## Usage

Run `om show` on any flake - via URL or local path.

```
$ om show github:srid/nixos-config
[..]
📦 Packages (nix build github:srid/nixos-config#<name>)
╭──────────┬───────────────────────────────────────────────────────╮
│ name     │ description                                           │
├──────────┼───────────────────────────────────────────────────────┤
│ activate │ Activate NixOS/nix-darwin/home-manager configurations │
│ update   │ Update the primary flake inputs                       │
│ default  │ Activate NixOS/nix-darwin/home-manager configurations │
╰──────────┴───────────────────────────────────────────────────────╯

🐚 Devshells (nix develop github:srid/nixos-config#<name>)
╭─────────┬──────────────────────────────────╮
│ name    │ description                      │
├─────────┼──────────────────────────────────┤
│ default │ Dev environment for nixos-config │
╰─────────┴──────────────────────────────────╯

🔍 Checks (nix flake check)
╭────────────┬─────────────╮
│ name       │ description │
├────────────┼─────────────┤
│ pre-commit │ N/A         │
╰────────────┴─────────────╯

🐧 NixOS Configurations (nixos-rebuild switch --flake github:srid/nixos-config#<name>)
╭────────────┬─────────────╮
│ name       │ description │
├────────────┼─────────────┤
│ gate       │ N/A         │
│ vixen      │ N/A         │
│ pureintent │ N/A         │
╰────────────┴─────────────╯

🍏 Darwin Configurations (darwin-rebuild switch --flake github:srid/nixos-config#<name>)
╭────────────┬─────────────╮
│ name       │ description │
├────────────┼─────────────┤
│ appreciate │ N/A         │
╰────────────┴─────────────╯

🔧 NixOS Modules
╭─────────┬─────────────╮
│ name    │ description │
├─────────┼─────────────┤
│ common  │ N/A         │
│ default │ N/A         │
╰─────────┴─────────────╯

🎨 Overlays
╭─────────┬─────────────╮
│ name    │ description │
├─────────┼─────────────┤
│ default │ N/A         │
╰─────────┴─────────────╯
```