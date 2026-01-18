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

## Style Resolution

The CSL backend resolves style names in two ways:

### 1. Registry Aliases (Recommended)

The styles listed above are **registry styles** with short, memorable aliases. These provide:
- Short names (e.g., `ieee` instead of `institute-of-electrical-and-electronics-engineers`)
- Accurate numeric vs author-date detection
- Superscript rendering for styles like Nature

### 2. Hayagriva Fallback

Any style available in [hayagriva's archive](https://github.com/typst/hayagriva) (80+ styles) can be used by its full name:

```toml
csl-style = "annual-reviews"           # Numeric style
csl-style = "american-physics-society" # Another numeric style
csl-style = "council-of-science-editors-author-date"  # Author-date
```

**Fallback limitations:**
- Citation format (numeric vs author-date) is detected automatically from CSL metadata
- **Superscript styles render as bracketed** (e.g., `[1]` instead of `¹`) — superscript cannot be detected from CSL metadata alone

For best results, use registry aliases when available. If you need a specific style not in the registry, the fallback will still format citations correctly, just without superscript support.

### Dependent Styles

Some CSL styles are "dependent" — they reference a parent style instead of defining their own formatting. These are not supported. If you encounter an error about a dependent style, use the parent style instead (e.g., use `nature` instead of `nature-communications`).

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

### Vancouver vs Vancouver-Superscript

Both Vancouver styles are numeric, but differ in citation rendering:

**`vancouver`** (bracketed):

```text
As shown in previous studies [1], the results [2,3] indicate...
```

**`vancouver-superscript`** (superscript):

```text
As shown in previous studies¹, the results²,³ indicate...
```

Use `vancouver` for most medical journals. Use `vancouver-superscript` when the journal specifically requires superscript numbers (less common).

### Alphanumeric

The alphanumeric style uses author-based labels instead of sequential numbers:

```toml
[preprocessor.bib]
bibliography = "refs.bib"
backend = "csl"
csl-style = "alphanumeric"
```

Output:

- Inline: `[Smi24]`, `[JL23]` (author-derived label + 2-digit year)
- Bibliography: `[Smi24] Smith, J. "Title." 2024.`

Label format:

- Single author: First 3 letters of surname + year (e.g., `[Smi24]`)
- Two authors: Initials of both authors + year (e.g., `[JL23]` for Jones & Lee)

This style is useful when you want readers to identify sources at a glance without flipping to the bibliography.

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
