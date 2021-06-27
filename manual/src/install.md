# Installation

**mdbook-bib** requires **mdbook** to be installed. Follow the instructions [here](https://github.com/rust-lang/mdBook#installation) to install **mdbook**.

Once mdbook is installed, if you are a [Rustacean](https://www.rust-lang.org/) and you have `Rust/Cargo` installed, you can get **mdbook-bib** from [Crates](https://crates.io/crates/mdbook-bib) simply with:

```sh
cargo install mdbook-bib
```

## Install from Binaries

1. Download and uncompress the [binaries](https://github.com/francisco-perez-sorrosal/mdbook-bib/releases) for your OS (Linux, Windows or macOS)
2. Add the parent directory of the executable to your `PATH` env variable to make the binary available in your shell

## Install from Sources

```sh
git clone git@github.com:francisco-perez-sorrosal/mdbook-bib.git
cd mdbook-bib
cargo install --path .
```

Make sure your `PATH` env var contains Cargo's `/bin` directory where the plugin was installed. 

Then follow the instructions [here](config.md) to use the plugin.
