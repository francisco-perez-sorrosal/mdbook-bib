# Configuration

## Adding a BibLaTex-format Bibliography

**mdbook-bib** allows adding a bibliography in [BibLaTex format](https://www.ctan.org/pkg/biblatex) to your book. 

Assuming that your directory structure for your book looks like this:

```
my_book/
├── book.toml
└── src
    ├── chapter_1.md
    └── SUMMARY.md
```

1. just add your `.bib` file containing the bibliography items to the root of your mdbook...

```
my_book/
├── book.toml
├── my_biblio.bib
└── src
    ├── chapter_1.md
    └── SUMMARY.md
```

2. ...and then add the following configuration to the `.toml` config file:

```toml
[book]
#...
[preprocessor.bib]
bibliography = "my_biblio.bib"
```

The bibliography will appear as a separate section in your book ToC. 

## Adding a BibLaTex-format Bibliography from [Zotero](https://www.zotero.org/)

Alternatively, you can use any publicly available library in BibLaTex format from Zotero.
In order to do so, just specify the `Zotero UserId` of the public bibliography you want to access in the preprocessor section:

```toml
[book]
#...
[preprocessor.bib]
zotero-uid = "<a_Zotero_userID>"
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


## Configuration Parameters

| Option           | Description                                                             | Default        |
|------------------|-------------------------------------------------------------------------|----------------|
| `bibliography`   | `.bib` file to use                                                      | -              |
| `zotero-uid`     | Zotero user ID, alternative to bib file                                 | -              |
| `title`          | Title for the Bibliography section of the book                          | `Bibliography` |
| `render-bib`     | Render the entire bibliography (`all`), or only cited entries (`cited`) | `cited`        |

A complete `preprocessor.bib` section example, which reads the bibliography from a local file and only shows the cited entries of the bibliography:

```toml
[preprocessor.bib]
title = "My Biblio!"
bibliography = "my_biblio.bib"
render-bib = "cited"
```
