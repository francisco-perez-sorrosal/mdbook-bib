# mdbook-bib

A [mdBook](https://github.com/rust-lang/mdBook) plugin for creating a bibliography & citations in your books.

[![Test status](https://github.com/francisco-perez-sorrosal/mdbook-bib/actions/workflows/test.yml/badge.svg)](https://github.com/francisco-perez-sorrosal/mdbook-bib/actions/workflows/test.yml)
[![MPL-2.0 License](https://img.shields.io/github/license/francisco-perez-sorrosal/mdbook-bib)](https://github.com/francisco-perez-sorrosal/mdbook-bib/blob/master/LICENSE)
[![Manual](https://img.shields.io/badge/book-master-blue.svg)](https://francisco-perez-sorrosal.github.io/mdbook-bib/)

[![Crate](https://img.shields.io/crates/v/mdbook-bib.svg)](https://crates.io/crates/mdbook-bib)
![Crates.io](https://img.shields.io/crates/d/mdbook-bib?style=social&link=https://crates.io/crates/mdbook-bib)

## Basic Install

If you have [mdbook](https://github.com/rust-lang/mdBook) installed just do:

```sh
cargo install mdbook-bib
```

Make sure your PATH env var contains Cargo's `/bin` directory where the plugin was installed. Then follow the instructions below to use the plugin.

See all options in the [Install section of the manual](https://francisco-perez-sorrosal.github.io/mdbook-bib/install.html).

## Add a BibLaTex File and Cite your Bib Entries!

Add a bibliography file in [BibLaTex format](https://www.ctan.org/pkg/biblatex) to the root of your book and then add the following section to the mdbook's `.toml` config file:

```toml
[book]
#...
[preprocessor.bib]
bibliography = "my_biblio.bib"
```

The bibliography will appear as a separate section in your book ToC.

Now you can add references/citations to the citation-keys appearing in the `.bib` file with:

```handlebars
{{#cite my-citation-key}}
```

See other configuration options in the [Config section of the manual](https://francisco-perez-sorrosal.github.io/mdbook-bib/config.html).

## Contribute

Check the [Contrib section of the manual](https://francisco-perez-sorrosal.github.io/mdbook-bib/contrib.html) if you want to contribute to mdbook-bib!
