[![project chat](https://img.shields.io/badge/zulip-join_chat-brightgreen.svg)](https://nixos.zulipchat.com/#narrow/stream/415454-omnix)

# omnix

<img width="10%" src="./doc/src/favicon.svg">

*Pronounced [`/ɒmˈnɪks/`](http://ipa-reader.xyz/?text=%C9%92m%CB%88n%C9%AAks&voice=Geraint)*

> [!NOTE]
> 🚧 omnix is in active development. It aims to supplement the [Nix](https://nixos.asia/en/nix) CLI to improve developer experience.

## Usage

See https://omnix.page/

## Developing

1. [Install Nix](https://nixos.asia/en/install)
1. [Setup `direnv`](https://nixos.asia/en/direnv)
1. Clone this repo, `cd` to it, and run `direnv allow`.

This will automatically activate the nix develop shell. Open VSCode and install recommended extensions, ensuring that direnv activates in VSCode as well.

> [!NOTE]
> If you would like to **learn the tools & technology** involved in this project, follow along this README noting the places where the emoji 🎓 is used.

### Running locally

To run `omnix-cli`,

```sh
just watch # Or `just w`; you can also pass args, e.g.: `just w show`
```

To run `omnix-gui`,

```sh
just watch-gui # Or `just wg`
```

`just watch-gui` runs `dx serve` (with hot reload disabled) that will restart the desktop app after compilation.

### Nix workflows

Inside the nix develop shell (activated by direnv) you can use any of the `cargo` or `rustc` commands, as well as [`just`](https://just.systems/) workflows. Nix specific commands can also be used to work with the project:

```sh
# Full nix build of CLI & GUI
nix build .#default .#gui

# Build and run the CLI
nix run
# Build and run the GUI
nix run .#gui
```

### Contributing

>[!TIP]
> Run `just fmt` to autoformat the source tree.

- Run `just ci` to **run CI locally**.
- Add **documentation** wherever useful.
    - Run `just doc run` to preview website docs; edit, and run `just doc check`
    - To preview Rust API docs, run `just doc cargo`.
- Changes to library crates must accompany a corresponding `CHANGELOG.md` entry.[^cc]

[^cc]: We don't use any automatic changelog generator for this repo.

### Tech

#### GUI app (`omnix-gui`)

We use [Dioxus](https://dioxuslabs.com/) to build the GUI using web technologies, as well as [dioxus-signals](https://github.com/DioxusLabs/dioxus/tree/master/packages/signals) for data reactivity.

##### Styling

We use [Tailwind](https://tailwindcss.com/) for styling; 🎓 familiarize yourself with it! Tailwind enables developers not familiar with design to create reasonably good looking sites. You should also 🎓 get familiar with CSS flexboxes (see [Flexbox Froggy](https://flexboxfroggy.com/)).

###### Color palette

See `tailwind.config.js` for colour aliases we use throughout the app. Instead of, say, `text-pink-500` we use `text-primary-500` ("primary" is more semantic than "pink").

### Crates

| Crate                               | Description                                   |
| ----------------------------------- | --------------------------------------------- |
| `./crates/nix_rs`                   | Rust interface to the Nix command line        |
| `./crates/nix_health`               | Nix health check library and executable       |
| `./crates/nixci`                    | Define and build CI for Nix projects anywhere |
| `./crates/flakreate`                | Rich flake templates                          |
| https://github.com/juspay/direnv-rs | Rust bindings for direnv                      |
