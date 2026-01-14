# mdbook-bib

[![GitHub](https://img.shields.io/badge/github-repo-blue?logo=github)](https://github.com/francisco-perez-sorrosal/mdbook-bib)
[![Crate](https://img.shields.io/crates/v/mdbook-bib.svg)](https://crates.io/crates/mdbook-bib)
[![Test status](https://github.com/francisco-perez-sorrosal/mdbook-bib/actions/workflows/test.yml/badge.svg)](https://github.com/francisco-perez-sorrosal/mdbook-bib/actions/workflows/test.yml)
[![MPL-2.0 License](https://img.shields.io/github/license/francisco-perez-sorrosal/mdbook-bib)](https://github.com/francisco-perez-sorrosal/mdbook-bib/blob/master/LICENSE)

**mdbook-bib** is a {{#cite mdBook}} plugin for creating a bibliography and referencing citations in your books. mdBook is written in the Rust language @@Klabnik2018.

## Two Rendering Backends

mdbook-bib offers two rendering backends to suit different needs:

| Feature | Custom (Handlebars) | CSL |
|---------|---------------------|-----|
| **Use case** | Full customization | Standard academic formats |
| **Citation styles** | Custom templates | IEEE, Chicago, Nature, APA, 80+ more |
| **Interactive elements** | Copy buttons, collapsible details | Basic (links only) |
| **Configuration** | More flexible | Simpler |

### Custom Backend (Default)

The **Custom backend** uses [Handlebars](https://handlebarsjs.com/) templates for maximum flexibility. You control exactly how citations and bibliography entries are rendered, including custom HTML, CSS, and JavaScript.

Best for: Power users who need custom layouts, interactive elements, or non-standard citation formats.

See [Custom Backend](./custom.md) for details.

### CSL Backend

The **CSL backend** uses [hayagriva](https://github.com/typst/hayagriva) to render citations in standard academic formats. Simply specify a style name and get properly formatted output.

Best for: Academic writing where you need standard citation styles like IEEE, APA, or Chicago.

See [CSL Backend](./csl.md) for details.

## Quick Start

```toml
[preprocessor.bib]
# Custom mode by default
bibliography = "refs.bib"
# For CSL mode, add:
# backend = "csl"
# csl-style = "ieee"
```

Cite entries with `{{#cite key}}` or `@@key`.

## Upgrading from v0.5.x

Version 1.0.0 introduces the dual backend system while maintaining full backwards compatibility. Your existing books will build without any changes. If you want to explore the new CSL backend or learn about new template variables, see the [Migration Guide](./migration.md).

### GitHub project

mdbook-bib is Open Source and available [on GitHub](https://github.com/francisco-perez-sorrosal/mdbook-bib).

### License

mdbook-bib is released under the [MPL-2.0 License](https://github.com/francisco-perez-sorrosal/mdbook-bib/blob/master/LICENSE).
