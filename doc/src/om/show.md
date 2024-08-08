# Show

The `om show` command seeks to provide a better `nix flake show` experience.

> [!WARNING]
> Currently, `om show` is a wrapper on `nix flake show`, but with support for [flake schemas](https://github.com/NixOS/nix/pull/8892). More is planned for `om show`. See [issue #162](https://github.com/juspay/omnix/issues/162).

## Usage

Run `om show` on any flake - via URL or local path.

```
$ om show github:srid/nixos-config
🐚 nix --extra-experimental-features 'nix-command flakes' show-config --json️
🐚 /nix/store/n02w2ybg9fc78grzz9i2aj49q3rysp7m-nix-2.24.0pre20240801_af10904/bin/nix flake show --legacy --allow-import-from-derivation --json --default-flake-schemas /nix/store/xzalq6mcw0ahyaccab6k98dbx3ll53y6-source github:srid/nixos-config️
📦 Packages (nix build github:srid/nixos-config#<name>)
╭──────────┬───────────────────────────────────────────────────────╮
│ name     │ description                                           │
├──────────┼───────────────────────────────────────────────────────┤
│ activate │ Activate NixOS/nix-darwin/home-manager configurations │
│ default  │ Activate NixOS/nix-darwin/home-manager configurations │
│ update   │ N/A                                                   │
╰──────────┴───────────────────────────────────────────────────────╯

🐚 Devshells (nix develop github:srid/nixos-config#<name>)
╭─────────┬──────────────────────────────────╮
│ name    │ description                      │
├─────────┼──────────────────────────────────┤
│ default │ Dev environment for nixos-config │
╰─────────┴──────────────────────────────────╯

🔍 Checks (nix flake check)
╭─────────┬─────────────╮
│ name    │ description │
├─────────┼─────────────┤
│ treefmt │ N/A         │
╰─────────┴─────────────╯

🐧 NixOS Configurations (nixos-rebuild switch --flake github:srid/nixos-config#<name>)
╭───────────┬─────────────╮
│ name      │ description │
├───────────┼─────────────┤
│ immediacy │ N/A         │
╰───────────┴─────────────╯

🍏 Darwin Configurations (darwin-rebuild switch --flake github:srid/nixos-config#<name>)
╭────────────┬─────────────╮
│ name       │ description │
├────────────┼─────────────┤
│ appreciate │ N/A         │
╰────────────┴─────────────╯

🔧 NixOS Modules
╭──────────────┬─────────────╮
│ name         │ description │
├──────────────┼─────────────┤
│ common       │ N/A         │
│ default      │ N/A         │
│ home-manager │ N/A         │
│ my-home      │ N/A         │
│ nixosFlake   │ N/A         │
╰──────────────┴─────────────╯
```