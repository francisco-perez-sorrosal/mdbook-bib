# Configuration

## Quick Start

Add a bibliography file to your mdbook source directory:

```
my_book/
├── book.toml
└── src
    ├── refs.bib
    ├── chapter_1.md
    └── SUMMARY.md
```

Configure in `book.toml`:

```toml
[preprocessor.bib]
bibliography = "refs.bib"
```

That's it! The bibliography appears as a section in your book's table of contents.

---

## Citation Syntax

Citation syntax controls **how you write citations** in your markdown files.

### Default Syntax

Use either the Handlebars-style or double-at notation:

| Pattern | Example | Description |
|---------|---------|-------------|
| `{{#cite key}}` | `{{#cite smith2024}}` | Handlebars-style |
| `@@key` | `@@smith2024` | Shorthand notation |

**Example:**

```markdown
According to {{#cite smith2024}}, the results show...
The experiment confirmed earlier findings @@jones2023.
```

### Pandoc Syntax (Opt-in)

Enable Pandoc-compatible syntax for cross-tool workflows (e.g., generating both HTML via mdBook and PDF via Pandoc):

```toml
[preprocessor.bib]
bibliography = "refs.bib"
citation-syntax = "pandoc"
```

| Pattern | Example | Meaning |
|---------|---------|---------|
| `@key` | `@smith2024 says...` | Author-in-text |
| `[@key]` | `This is true [@smith2024]` | Parenthetical |
| `[-@key]` | `Smith says [-@smith2024]` | Suppress author (year only) |
| `\@` | `user\@example.com` | Literal @ (escaped) |

This lets you use the same source files with both mdBook (HTML) and Pandoc (PDF).

**Example:**

```markdown
According to @smith2024, the results show significant improvement.
This has been documented [@jones2023].
Jones argues [-@jones2023] that further research is needed.
Contact: user\@example.com
```

**Notes:**
- Default syntax (`{{#cite key}}` and `@@key`) continues to work alongside Pandoc syntax
- Citations inside code blocks are NOT processed
- Email addresses and URL mentions (e.g., `https://twitter.com/@user`) are NOT matched

**Citation key format:**
- Native patterns (`{{#cite}}`, `@@`): Allow keys starting with digits (e.g., `@@2024smith`) — BibLaTeX-compliant
- Pandoc patterns (`@key`, `[@key]`, `[-@key]`): Require letter or underscore at start (e.g., `@smith2024`) — Pandoc spec

This difference is by design for cross-tool compatibility. If you need digit-prefixed keys with Pandoc syntax, use the native `{{#cite 123key}}` form instead.

**Unsupported Pandoc features:**
- Multi-citation: `[@smith2024; @jones2023]` — use separate citations instead
- Locators: `[@smith2024, p. 42]` — page numbers not supported
- Prefixes/suffixes: `[see @smith2024, ch. 3]` — use surrounding text instead

---

## Backend Selection

The backend controls **how citations are rendered** in the output.

### Understanding the Pipeline

```
┌─────────────────────────────────────────────────────────────────┐
│                    YOUR MARKDOWN SOURCE                         │
│  "According to @smith2024, this is true [@jones2023]."          │
└─────────────────────────┬───────────────────────────────────────┘
                          │
                          ▼
              ┌───────────────────────┐
              │   citation-syntax     │  ← How citations are WRITTEN
              │   "default" | "pandoc"│     (input parsing)
              └───────────┬───────────┘
                          │
                          ▼
              ┌───────────────────────┐
              │       backend         │  ← How citations are RENDERED
              │   "custom" | "csl"    │     (output engine)
              └───────────┬───────────┘
                          │
            ┌─────────────┴─────────────┐
            ▼                           ▼
   ┌─────────────────┐        ┌─────────────────┐
   │ backend="custom"│        │  backend="csl"  │
   │                 │        │                 │
   │ Handlebars      │        │ CSL styles via  │
   │ templates       │        │ hayagriva       │
   └─────────────────┘        │  • csl-style    │
                              └─────────────────┘
```

### Custom Backend (Default)

Uses Handlebars templates for full customization of citation and bibliography appearance.

```toml
[preprocessor.bib]
bibliography = "refs.bib"
# backend = "custom"  # This is the default
```

See [Custom Backend](./custom.md) for template options.

### CSL Backend

Uses [hayagriva](https://github.com/typst/hayagriva) for standardized academic citation styles (IEEE, APA, Chicago, etc.).

```toml
[preprocessor.bib]
bibliography = "refs.bib"
backend = "csl"
csl-style = "ieee"
```

See [CSL Backend](./csl.md) for available styles.

---

## Bibliography Options

### Section Title

```toml
[preprocessor.bib]
title = "References"  # Default: "Bibliography"
```

### Which Entries to Show

```toml
[preprocessor.bib]
render-bib = "cited"  # Only show cited entries (default)
render-bib = "all"    # Show all entries from the bibliography file
```

### Sort Order

```toml
[preprocessor.bib]
order = "none"    # Original order from file (default)
order = "key"     # Alphabetical by citation key
order = "author"  # Alphabetical by author name
order = "index"   # Order of first citation in the book
```

### Per-Chapter Bibliographies

Add a bibliography section at the end of each chapter:

```toml
[preprocessor.bib]
add-bib-in-chapters = true  # Default: false
```

---

## Configuration Reference

| Option | Description | Default |
|--------|-------------|---------|
| **Source** | | |
| `bibliography` | Path to `.bib` or `.yaml` file | - |
| `zotero-uid` | Zotero user ID (alternative to file) | - |
| **Citation Syntax** | | |
| `citation-syntax` | Input syntax: `default` or `pandoc` | `default` |
| **Backend** | | |
| `backend` | Rendering engine: `custom` or `csl` | `custom` |
| `csl-style` | CSL style name (when `backend = "csl"`) | - |
| **Bibliography Output** | | |
| `title` | Bibliography section title | `Bibliography` |
| `render-bib` | Show `all` entries or only `cited` | `cited` |
| `order` | Sort: `none`, `key`, `author`, `index` | `none` |
| `add-bib-in-chapters` | Add bibliography per chapter | `false` |
| **Custom Backend Templates** | | |
| `hb-tpl` | Handlebars template for entries | - |
| `cite-hb-tpl` | Handlebars template for citations | - |
| `css` | Custom CSS file | - |
| `js` | Custom JavaScript file | - |

---

## Examples

### Minimal Setup

```toml
[preprocessor.bib]
bibliography = "refs.bib"
```

### Academic Paper with IEEE Style

```toml
[preprocessor.bib]
bibliography = "refs.bib"
backend = "csl"
csl-style = "ieee"
title = "References"
```

### Pandoc-Compatible Workflow

```toml
[preprocessor.bib]
bibliography = "refs.bib"
citation-syntax = "pandoc"
backend = "csl"
csl-style = "apa"
```

### Fully Customized

```toml
[preprocessor.bib]
bibliography = "refs.bib"
title = "Bibliography"
render-bib = "cited"
order = "index"
add-bib-in-chapters = true
hb-tpl = "templates/references.hbs"
cite-hb-tpl = "templates/citation.hbs"
css = "templates/style.css"
```

---

## Advanced Options

### Using Zotero

Download a public bibliography from [Zotero](https://www.zotero.org/) instead of a local file:

```toml
[preprocessor.bib]
zotero-uid = "<your_zotero_user_id>"
```

**Setup:**
1. Find your User ID in [Zotero Feeds/API settings](https://www.zotero.org/settings/keys)
2. Make your library public in [Privacy Settings](https://www.zotero.org/settings/privacy)

### Debugging

Enable debug logging to troubleshoot issues:

```bash
MDBOOK_LOG=mdbook_bib=debug mdbook build
```

For more verbose output:

```bash
MDBOOK_LOG=debug mdbook build
```
