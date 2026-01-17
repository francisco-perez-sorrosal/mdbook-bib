# CSL Backend (hayagriva)

The CSL backend uses [hayagriva](https://github.com/typst/hayagriva) to render citations and bibliographies in standard academic formats. This provides properly formatted output for IEEE, Chicago, APA, Nature, and 80+ other citation styles.

## When to Use CSL Backend

- You need standard academic citation formatting
- You want numbered citations (IEEE, Vancouver) or author-date citations (Chicago, APA)
- You need superscript citations (Nature)
- You want consistent formatting without manual template work

## Configuration

Enable the CSL backend by setting `backend = "csl"`:

```toml
[preprocessor.bib]
bibliography = "refs.bib"
backend = "csl"
csl-style = "ieee"
```

## Supported Styles

### Numeric Styles

Citations appear as numbers in brackets or superscripts:

| Style | Citation Format | Example |
|-------|-----------------|---------|
| `ieee` | `[1]`, `[2]` | IEEE transactions |
| `vancouver` | `[1]`, `[2]` | Medical journals |
| `vancouver-superscript` | `¹`, `²` (superscript) | Vancouver superscript variant |
| `nature` | `¹`, `²` (superscript) | Nature journal |
| `acm` | `[1]`, `[2]` | ACM publications |
| `acs` | `[1]`, `[2]` | American Chemical Society |
| `ama` | `[1]`, `[2]` | American Medical Association |
| `cell` | `[1]`, `[2]` | Cell journal |
| `springer-basic` | `[1]`, `[2]` | Springer publications |
| `elsevier-vancouver` | `[1]`, `[2]` | Elsevier Vancouver |
| `alphanumeric` | `[Smi24]` | Alphanumeric labels |

### Author-Date Styles

Citations include author name and year:

| Style | Citation Format | Example |
|-------|-----------------|---------|
| `chicago-author-date` | (Author Year) | Chicago Manual of Style |
| `apa` | (Author, Year) | APA 7th edition |
| `mla` | (Author Page) | MLA 9th edition |
| `mla8` | (Author Page) | MLA 8th edition |
| `harvard` | (Author Year) | Harvard referencing |
| `elsevier-harvard` | (Author, Year) | Elsevier journals |
| `springer-basic-author-date` | (Author Year) | Springer author-date |

### Note Styles

Some styles use footnotes or endnotes:

| Style | Format |
|-------|--------|
| `chicago-notes` | Footnote citations |

## Examples by Style

### IEEE (Numeric)

```toml
[preprocessor.bib]
bibliography = "refs.bib"
backend = "csl"
csl-style = "ieee"
```

Output:
- Inline: `[1]`, `[2]`, `[3]`
- Bibliography: `[1] A. Author, "Title," Journal, vol. 1, pp. 1-10, 2024.`

### Chicago Author-Date

```toml
[preprocessor.bib]
bibliography = "refs.bib"
backend = "csl"
csl-style = "chicago-author-date"
```

Output:
- Inline: `(Smith 2024)`, `(Jones and Lee 2023)`
- Bibliography: `Smith, John. 2024. "Title." Journal 1: 1-10.`

### Nature (Superscript)

```toml
[preprocessor.bib]
bibliography = "refs.bib"
backend = "csl"
csl-style = "nature"
```

Output:
- Inline: `¹`, `²`, `³` (as superscripts)
- Bibliography: `1. Author, A. Title. Journal 1, 1-10 (2024).`

### APA

```toml
[preprocessor.bib]
bibliography = "refs.bib"
backend = "csl"
csl-style = "apa"
```

Output:
- Inline: `(Author, 2024)`
- Bibliography: `Author, A. (2024). Title. Journal, 1, 1-10.`

## Full Configuration Example

```toml
[preprocessor.bib]
bibliography = "references.bib"
backend = "csl"
csl-style = "chicago-author-date"
title = "References"
render-bib = "cited"    # Only show cited entries
order = "author"        # Sort by author name
```

## YAML Bibliography Support

The CSL backend also supports YAML bibliography files (hayagriva's native format):

```yaml
# refs.yaml
smith2024:
  type: article
  title: Example Article
  author:
    - Smith, John
    - Jones, Jane
  date: 2024
  parent:
    - type: periodical
      title: Journal of Examples
      volume: 1
      page: 1-10
```

```toml
[preprocessor.bib]
bibliography = "refs.yaml"
backend = "csl"
csl-style = "apa"
```

## Citation Linking

Citations automatically link to their bibliography entries. The bibliography page includes anchor IDs matching the citation keys.

## Limitations

The CSL backend does not support:

- Custom HTML templates (use Custom backend for this)
- Copy buttons or interactive elements
- Custom JavaScript integration

For these features, use the [Custom Backend](./custom.md).

## Tips

- Choose numeric styles (IEEE, Vancouver) for technical papers
- Choose author-date styles (Chicago, APA) for humanities and social sciences
- Use Nature style for scientific journals requiring superscript citations
- Check your target journal's requirements before selecting a style
