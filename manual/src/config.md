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

1. just add your `.bib` file containing the bibliography items to the root source of your mdbook (pointed by the `src` parameter in the `[book]` section of the `.toml` file)...

```
my_book/
├── book.toml
└── src
    ├── my_biblio.bib
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

In your markdown files, create references/citations to the citation-keys included in the `.bib` file with any of these two options:

1. Surround the citation key with the `{{#cite` and `}}` delimiters
2. Prepend the citation key with two `@` characters

## Configure your own Style for Bibliography Entries

You can override the default biblio style provided for the biliography entries by specifying an ad-hoc Handlebars template and style. In order to do so, the `hb-tpl`, `css`, and `js` parameters are provided as configuration options. `hb-tpl` allows to point to a `.hbs` file that includes the Handlebars style. For example:

```handlebars
{{#include render/my_references.hbs}}
```

The available placeholders that can be used in the handlebars template for now are:

* citation_key
* authors
* title
* url
* pub_year

Also, with the parameters `css` and `js`, you can point to files that provide your own css style and/or Javascript functions used in the rendering of the Handlebars template entries (e.g. for the `bib_div` class above). For more details, check the [structure of the manual](https://github.com/francisco-perez-sorrosal/mdbook-bib/tree/master/manual) of this project.

## Configuration Parameters

| Option           | Description                                                             | Default |
|------------------|-------------------------------------------------------------------------|---------|
| `bibliography`   | `.bib` file to use.                                                     | -       |
| `zotero-uid`     | Zotero user ID, alternative to bib file.                                | -       |
| `title`          | Title for the Bibliography section of the book                          | `Bibliography` |
| `render-bib`     | Render the entire bibliography (`all`), or only cited entries (`cited`) | `cited` |
| `hb-tpl`         | Ad-hoc Handlebars template file used to render the bibliography. Overwrites the default style.                                                                                       | -       |
| `css`            | Extra CSS file with the style used when rendering the ad-hoc biblio.    | -       |
| `js`             | Extra JS file with code used when rendering the ad-hoc biblio.          | -       |

A complete `preprocessor.bib` section example, which reads the bibliography from a local file and only shows the cited entries of the bibliography:

```toml
[preprocessor.bib]
title = "My Biblio!"
bibliography = "my_biblio.bib"
render-bib = "cited"
hb-tpl = "render/my_references.hbs"
css = "render/my_style.css"
```
