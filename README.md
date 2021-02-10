# mdbook plugin for adding a Bibliography & Citations
                                               
# Manual Install

```sh
git clone git@github.com:francisco-perez-sorrosal/mdbook-bib.git
cd mdbook-bib
cargo install --path .
```

Make sure your PATH env var contains Cargo's /bin directory where the plugin was intalled.
Then follow the instructions below to use the plugin.

TODO: Deploy a package in crates.io

## Adding a BibText-format Bibliography

This pluging allows adding a bibliography in [BibTex format](http://www.bibtex.org/Format/) to your
book. To do this, just add your `.bib` file containing the bibliography items to the
source root of your book and then add the following configuration to the `book`
section of the `.toml` config file of your mdbook:

```toml
[book]
...
[preprocessor.bib]
bibliography = "my_biblio.bib"
renderer = ["html"]
```

The bibliography will appear as a separate section in your book ToC. 

## Add References/Citations to the Bibliography

You can create references/citations to the citation-keys included in the `.bib` file in your markdown files
with the following syntax:

```hbs
{{#cite my-citation-key}}