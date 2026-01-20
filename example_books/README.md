# Example Books

This directory contains working examples demonstrating different mdbook-bib configurations.

## Building Examples

To build any example:

```sh
cd example_books/<example-name>
mdbook build && mdbook serve
```

## Examples

### basic

Custom backend with Handlebars templates and CSS styling. Demonstrates:

- Custom citation template (`src/render/my_cite.hbs`)
- Custom bibliography template (`src/render/my_references.hbs`)
- Custom CSS (`src/render/my_style.css`)
- Nested chapter structure with proper relative paths

### csl_ieee

CSL backend with IEEE citation style. Produces numbered citations like `[1]`, `[2]`.

### csl_chicago

CSL backend with Chicago author-date style. Produces citations like `(Smith 2024)`.

### csl_nature

CSL backend with Nature citation style. Produces superscript citations like `¹`, `²`.

### csl_alphanumeric

CSL backend with alphanumeric labels. Produces citations like `[Smi24]` based on author name and year.

### pandoc

Pandoc-compatible citation syntax (`@key`, `[@key]`, `[-@key]`). Useful for workflows that generate both HTML (mdBook) and PDF (Pandoc) from the same sources.

### templates

Alternative Handlebars templates for the Custom backend:

- `cite_key.hbs` - Citation by key (markdown link style)
- `cite_index.hbs` - Citation by index number `[1]`
- `cite_author_year.hbs` - Citation by author and year
- `refs_default.hbs` - Expandable reference entries with copy button
- `refs_index.hbs` - References sorted by citation index

Copy these or your own to your book's `src/` directory and configure in `book.toml`:

```toml
[preprocessor.bib]
cite-hb-tpl = "templates/cite_index.hbs"
hb-tpl = "templates/refs_default.hbs"
```
