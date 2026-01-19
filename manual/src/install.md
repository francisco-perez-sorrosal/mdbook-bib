# Installation

**mdbook-bib** requires [mdbook](https://github.com/rust-lang/mdBook#installation) to be installed.

## Install with Cargo (Recommended)

Once mdbook is installed, if you are a [Rustacean](https://www.rust-lang.org/) and you have `Rust/Cargo` installed, you can get **mdbook-bib** from [Crates](https://crates.io/crates/mdbook-bib) simply with:

```sh
cargo install mdbook-bib
```

## Install from Binaries

1. Download the [binary](https://github.com/francisco-perez-sorrosal/mdbook-bib/releases) for your OS (Linux, Windows, macOS)
2. Add the executable's directory to your `PATH`

## Install from Source

```sh
git clone git@github.com:francisco-perez-sorrosal/mdbook-bib.git
cd mdbook-bib
cargo install --path .
```

Ensure Cargo's `bin/` directory is in your `PATH`. Then see [Configuration](config.md) to start using the plugin.
