# mdbook plugin for adding a Bibliography & Citations

## Install

```sh
cargo install mdbook-bib
```

## Install from Sources

```sh
git clone git@github.com:francisco-perez-sorrosal/mdbook-bib.git
cd mdbook-bib
cargo install --path .
```

Make sure your PATH env var contains Cargo's /bin directory where the plugin was intalled. Then follow the instructions
below to use the plugin.

## Adding a BibLaTex-format Bibliography

This plugin allows adding a bibliography in [BibLaTex format](https://www.ctan.org/pkg/biblatex) to your book. To do this,
just add your `.bib` file containing the bibliography items to the source root of your book and then add the following
configuration to the `book`
section of the `.toml` config file of your mdbook:

```toml
[book]
#...
[preprocessor.bib]
bibliography = "my_biblio.bib"
renderer = ["html"]
```

The bibliography will appear as a separate section in your book ToC. 

## Adding a BibLaTex-format Bibliography from Zotero

Alternatively, you can download a publicly available library in BibLaTex format from Zotero.
In order to do so, just specify the `Zotero UserId` of the public bibliography you want to access in the preprocessor 
section: 

```toml
[book]
#...
zotero_user_id = <a_Zotero_userID>
renderer = ["html"]
```

The `Zotero UserId` is the number that appears following the `users` resource in a public bibliography URL. e.g. in the 
example below, the `Zotero UserId` is 475425:
```shell
https://api.zotero.org/users/475425/items?format=atom&v=3
```

If you have a Zotero account, you can make your library public marking the checkbox in the [Zotero Privacy Settings page](https://www.zotero.org/settings/privacy).

You can find your `Zotero userID` in the [Zotero Feeds/API](https://www.zotero.org/settings/keys) section of your 
Zotero account.

## Add References/Citations to the Bibliography

In your markdown files, create references/citations to the citation-keys included in the `.bib` file with the 
following syntax:

```handlebars
{{#cite my-citation-key}}
```