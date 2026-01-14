# Migration Guide

This guide helps you upgrade from mdbook-bib v0.5.x to v1.0.0.

## What's New in v1.0.0

- **Dual backend system**: Choose between Custom (Handlebars) or CSL rendering
- **hayagriva-powered parsing**: Better BibTeX/BibLaTeX handling, YAML support
- **80+ CSL styles**: IEEE, Chicago, Nature, APA, and many more
- **Expanded bibliography fields**: DOI, keywords, volume, issue, and more

## Backwards Compatibility

**Good news**: Your existing books will work without any changes.

- The Custom backend is the default
- All existing configuration options work identically
- Custom Handlebars templates continue to work
- Citation syntax (`{{#cite key}}` and `@@key`) is unchanged

## Upgrading

### Step 1: Update mdbook-bib

```bash
cargo install mdbook-bib
```

### Step 2: Build Your Book

```bash
mdbook build
```

That's it. Your book should build exactly as before.

## Trying the CSL Backend

If you want standard academic citation formatting, try the CSL backend:

```toml
[preprocessor.bib]
bibliography = "refs.bib"
backend = "csl"
csl-style = "ieee"
```

See [CSL Backend](./csl.md) for available styles.

## Which Backend Should I Use?

### Use Custom Backend If You Need

- Custom HTML layouts for bibliography entries
- Interactive elements (copy buttons, tooltips, collapsible sections)
- Non-standard citation formats
- Full control over styling via CSS/JS

See [Custom Backend](./custom.md) for template customization.

### Use CSL Backend If You Need

- Standard academic citation styles (IEEE, APA, Chicago, Nature)
- Minimal configuration
- Consistent formatting matching journal requirements
- Superscript citations (Nature style)

## New Template Variables

If you use custom Handlebars templates, new fields are now available:

| Field | Description |
|-------|-------------|
| `entry_type` | article, book, inproceedings, etc. |
| `doi` | Digital Object Identifier |
| `pages` | Page numbers |
| `volume` | Volume number |
| `issue` | Issue number |
| `publisher` | Publisher name |
| `address` | Publisher location |
| `editor` | Editors (same format as authors) |
| `edition` | Edition |
| `series` | Series name |
| `note` | Additional notes |

These fields are optional. Use `{{#if field}}...{{/if}}` to conditionally render them:

```handlebars
{{#if doi}}
  <a href="https://doi.org/{{doi}}">DOI: {{doi}}</a>
{{/if}}
```

## YAML Bibliography Support

You can now use YAML format for bibliographies (hayagriva's native format):

```yaml
# refs.yaml
smith2024:
  type: article
  title: Example Article
  author: Smith, John
  date: 2024
```

```toml
[preprocessor.bib]
bibliography = "refs.yaml"
```

Both BibTeX and YAML work with either backend.

## Troubleshooting

### Citations Not Rendering

Enable debug logging:

```bash
MDBOOK_LOG=mdbook_bib=debug mdbook build
```

### CSL Style Not Found

Check the style name matches one of the [supported styles](./csl.md#supported-styles). Common names:

- `ieee` (not `IEEE`)
- `chicago-author-date` (not `chicago`)
- `apa` (not `APA`)

### Custom Templates Broken

If your templates stopped working, check for:

1. Template syntax errors (Handlebars is strict)
2. Changed field names (unlikely, but check debug output)

The new fields are additive—existing templates should work unchanged.

## Getting Help

- [GitHub Issues](https://github.com/francisco-perez-sorrosal/mdbook-bib/issues)
- Debug with `MDBOOK_LOG=mdbook_bib=debug mdbook build`
