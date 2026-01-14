# mdbook-bib

A [mdBook](https://github.com/rust-lang/mdBook) plugin for creating a bibliography & citations in your books.

[![Test status](https://github.com/francisco-perez-sorrosal/mdbook-bib/actions/workflows/test.yml/badge.svg)](https://github.com/francisco-perez-sorrosal/mdbook-bib/actions/workflows/test.yml)
[![MPL-2.0 License](https://img.shields.io/github/license/francisco-perez-sorrosal/mdbook-bib)](https://github.com/francisco-perez-sorrosal/mdbook-bib/blob/master/LICENSE)
[![Manual](https://img.shields.io/badge/book-master-blue.svg)](https://francisco-perez-sorrosal.github.io/mdbook-bib/)

[![Crate](https://img.shields.io/crates/v/mdbook-bib.svg)](https://crates.io/crates/mdbook-bib)
![Crates.io](https://img.shields.io/crates/d/mdbook-bib?style=social&link=https://crates.io/crates/mdbook-bib)

## Features

- Add citations from your BibLaTeX or YAML bibliography files
- Automatically download your public bibliography from Zotero and cite it
- **Two rendering backends**:
  - **Custom (Handlebars)**: Full template customization with custom CSS/JS
  - **CSL (Citation Style Language)**: Standard academic citation styles (IEEE, Chicago, Nature, APA, and 80+ more)

## Basic Install

If you have [mdbook](https://github.com/rust-lang/mdBook) installed just do:

```sh
cargo install mdbook-bib
```

Make sure your PATH env var contains Cargo's `/bin` directory where the plugin was installed. Then follow the instructions below to use the plugin.

See all options in the [Install section of the manual](https://francisco-perez-sorrosal.github.io/mdbook-bib/install.html).

## Add a Bibliography and Cite your Entries

Add a bibliography file in [BibLaTeX format](https://www.ctan.org/pkg/biblatex) (or YAML) to the root of your mdbook and configure the plugin in your `book.toml`:

```toml
[preprocessor.bib]
bibliography = "my_biblio.bib"
```

Now you can cite entries using either syntax:

```markdown
{{#cite my-citation-key}}
@@my-citation-key
```

## Rendering Backends

mdbook-bib provides two rendering backends. The **Custom backend** (default) gives you full control over citation and bibliography formatting through Handlebars templates, custom CSS, and JavaScript. The **CSL backend** uses [hayagriva](https://github.com/typst/hayagriva) to render citations in standard academic formats (IEEE, APA, Chicago, Nature, and 80+ more) without any template configuration.

Choose Custom if you need custom layouts or interactive elements. Choose CSL if you want standard academic formatting with minimal setup.

### Custom Backend (Default)

The default backend uses Handlebars templates for full customization:

```toml
[preprocessor.bib]
bibliography = "my_biblio.bib"
# Optional: custom templates
hb-tpl = "render/references.hbs"
cite-hb-tpl = "render/citation.hbs"
css = "render/style.css"
```

See [Custom Backend documentation](https://francisco-perez-sorrosal.github.io/mdbook-bib/custom.html) for template variables and examples.

### CSL Backend

For standard academic citation styles, enable the CSL backend:

```toml
[preprocessor.bib]
bibliography = "my_biblio.bib"
backend = "csl"
csl-style = "ieee"  # or: chicago-author-date, nature, apa, mla, harvard, ...
```

See [CSL Backend documentation](https://francisco-perez-sorrosal.github.io/mdbook-bib/csl.html) for available styles and examples.

See the [manual](https://francisco-perez-sorrosal.github.io/mdbook-bib/) for all configuration options.

**Tip**: Debug builds with `MDBOOK_LOG=mdbook_bib=debug mdbook build`.

## Contribute

Check the [Contrib section of the manual](https://francisco-perez-sorrosal.github.io/mdbook-bib/contrib.html) if you want to contribute to mdbook-bib!
