# Pandoc Citation Syntax Demo

This test book demonstrates Pandoc-compatible citation syntax in mdbook-bib.

## Configuration

This book is configured with:

```toml
[preprocessor.bib]
bibliography = "refs.bib"
citation-syntax = "pandoc"
backend = "csl"
csl-style = "apa"
```

## What This Enables

With `citation-syntax = "pandoc"`, you can use:

| Pattern | Purpose | Example |
|---------|---------|---------|
| `@key` | Author-in-text | @Klabnik2018 explains... |
| `[@key]` | Parenthetical | ...is well documented [@Matsakis2014]. |
| `[-@key]` | Year only | Klabnik says [-@Klabnik2018] that... |
| `\@` | Literal @ | Contact: user\@example.com |

The existing syntaxes (`{{#cite key}}` and `@@key`) continue to work alongside Pandoc syntax.

## Quick Example

According to @Klabnik2018, Rust provides memory safety without garbage collection.
This approach has been formally verified [@Jung2017].
