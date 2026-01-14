# Configuration

## Adding a Bibliography

**mdbook-bib** supports bibliographies in [BibLaTeX format](https://www.ctan.org/pkg/biblatex) or YAML format.

Add your `.bib` or `.yaml` file to your mdbook source directory:

```
my_book/
├── book.toml
└── src
    ├── refs.bib
    ├── chapter_1.md
    └── SUMMARY.md
```

Then configure in `book.toml`:

```toml
[preprocessor.bib]
bibliography = "refs.bib"
```

The bibliography appears as a separate section in your book's table of contents.

## Using Zotero

Alternatively, download a public bibliography from [Zotero](https://www.zotero.org/):

```toml
[preprocessor.bib]
zotero-uid = "<your_zotero_user_id>"
```

Find your User ID in [Zotero Feeds/API settings](https://www.zotero.org/settings/keys). Your library must be public in [Privacy Settings](https://www.zotero.org/settings/privacy).

## Adding Citations

Reference entries in your markdown using either syntax:

```markdown
According to {{#cite smith2024}}, the results show...
The experiment confirmed earlier findings @@jones2023.
```

## Backend Selection

mdbook-bib offers two rendering backends:

### Legacy (Default)

Uses Handlebars templates for full customization:

```toml
[preprocessor.bib]
bibliography = "refs.bib"
# backend = "legacy"  # Optional, this is the default
```

See [Legacy Backend](./legacy.md) for template customization options.

### CSL

Uses hayagriva for standard academic citation styles:

```toml
[preprocessor.bib]
bibliography = "refs.bib"
backend = "csl"
csl-style = "ieee"  # or: chicago-author-date, nature, apa, mla, ...
```

See [CSL Backend](./csl.md) for available styles.

## Configuration Reference

| Option | Description | Default |
|--------|-------------|---------|
| `bibliography` | Path to `.bib` or `.yaml` file | - |
| `zotero-uid` | Zotero user ID (alternative to file) | - |
| `backend` | Rendering backend: `legacy` or `csl` | `legacy` |
| `csl-style` | CSL style name (when `backend = "csl"`) | - |
| `title` | Bibliography section title | `Bibliography` |
| `render-bib` | Show `all` entries or only `cited` | `cited` |
| `order` | Sort order: `none`, `key`, `author`, `index` | `none` |
| `add-bib-in-chapters` | Add bibliography at end of each chapter | `false` |
| `hb-tpl` | Custom Handlebars template for entries | - |
| `cite-hb-tpl` | Custom Handlebars template for citations | - |
| `css` | Custom CSS file | - |
| `js` | Custom JavaScript file | - |

## Complete Examples

### Minimal (Legacy)

```toml
[preprocessor.bib]
bibliography = "refs.bib"
```

### Academic Paper (CSL)

```toml
[preprocessor.bib]
bibliography = "refs.bib"
backend = "csl"
csl-style = "ieee"
title = "References"
render-bib = "cited"
```

### Custom Templates (Legacy)

```toml
[preprocessor.bib]
bibliography = "refs.bib"
title = "Bibliography"
render-bib = "cited"
order = "index"
hb-tpl = "render/references.hbs"
cite-hb-tpl = "render/citation.hbs"
css = "render/style.css"
js = "render/script.js"
```

## Debugging

Enable debug logging for troubleshooting:

```bash
MDBOOK_LOG=mdbook_bib=debug mdbook build
```
