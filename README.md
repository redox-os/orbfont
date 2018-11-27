# OrbFont
Orbital font rendering. Compatible with Redox and SDL2.

[![Build status](https://gitlab.redox-os.org/redox-os/orbfont/badges/master/build.svg)](https://gitlab.redox-os.org/redox-os/orbfont/pipelines)
[![MIT licensed](https://img.shields.io/badge/license-MIT-blue.svg)](./LICENSE)
[![crates.io](https://img.shields.io/badge/crates.io-v0.1.8-orange.svg)](https://crates.io/crates/orbfont)
[![docs.rs](https://docs.rs/orbfont/badge.svg)](https://docs.rs/orbfont)


## Usage

To include OrbFont in your project, just add the dependency
line to your `Cargo.toml` file:

```text
orbfont = "0.1.8"
```

To use OrbFont master, just add the dependency
line to your `Cargo.toml` file:

```text
orbfont = { git = https://gitlab.redox-os.org/redox-os/orbfont.git }
```

However you also need to have the SDL2 libraries installed on your
system.  The best way to do this is documented [by the SDL2
crate](https://github.com/AngryLawyer/rust-sdl2#user-content-requirements).

## Examples

You find the examples in the `examples/` directory.

You can start the widgets example by executing the following command:

```text
cargo run --example character_map --release
```

## Build and run documenation

You can build and run the latest documentation by executing the following command:

```text
cargo doc --no-deps --open
```