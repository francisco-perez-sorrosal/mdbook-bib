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
- **Pandoc-compatible citation syntax** for cross-tool workflows (generate HTML with mdBook and PDF with Pandoc from the same source)
- **Two rendering backends**:
  - **Custom (Handlebars)**: Full template customization with custom CSS/JS
  - **CSL (Citation Style Language)**: Standard academic citation styles (IEEE, Chicago, Nature, APA, and 80+ more)
  
## TL;DR

Create an example mdbook:

```sh
cargo install mdbook mdbook-bib
mdbook init mybook && cd mybook
```

Add mdbook-bib config to `book.toml`:

```toml
[preprocessor.bib]
bibliography = "refs.bib"
```

Create example bibliography `refs.bib`:

```bibtex
@article{hello,
  author = {World, Hello},
  title = {My First Citation},
  year = {2024}
}
```

Cite in `src/chapter_1.md`:

```markdown
As shown in @@hello, citations are easy!
```

Build book: `mdbook build`

Serve the book: `mdbook serve`

Then open http://localhost:3000 in your browser to view your book.


## Install

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

### Pandoc-Compatible Syntax

Enable Pandoc citation syntax for cross-tool workflows:

```toml
[preprocessor.bib]
...
citation-syntax = "pandoc"
```

Then use standard Pandoc citations:

```markdown
@key              # Author-in-text: "Smith (2024)"
[@key]            # Parenthetical: "(Smith, 2024)"
[-@key]           # Suppress author: "(2024)"
```

This lets you use the same source files with both mdBook (HTML) and Pandoc (PDF).

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
