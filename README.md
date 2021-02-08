# mdbook plugin for adding a Bibliography & Citations

## Adding a BibText-format Bibliography

This pluging allows adding a bibliography in [BibTex format](http://www.bibtex.org/Format/) to your
book. To do this, just add your `.bib` file containing the bibliography items to the
source root of your book and then add the following configuration to the `book`
section of the `.toml` config file of your mdbook:

```toml
[book]
...
bibliography = "my_biblio.bib"
...
```

The bibliography will appear as a separate section in your book ToC. 

## Add References/Citations to the Bibliography

You can create references/citations to the citation-keys included in the `.bib` file in your markdown files
with the following syntax:

```hbs
\{{#cite my-citation-key}}