# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

mdbook-bib is a Rust-based mdBook preprocessor plugin that enables bibliography management and citations in mdBook projects. It supports BibLaTeX format bibliographies and Zotero integration.

## Build, Test, and Development Commands

### Building
```bash
cargo build              # Build the project
cargo build --release    # Build optimized release version
```

### Testing
```bash
cargo test              # Run all tests
cargo test --workspace  # Run tests for the entire workspace
```

### Code Quality
```bash
cargo fmt --            # Format all code
cargo clippy --fix      # Run clippy with auto-fixes
cargo clippy --fix --tests  # Run clippy with auto-fixes for tests
```

**Important**: Pre-commit hooks in `.rusty-hook.toml` automatically enforce formatting and clippy checks. Commits will be blocked if formatting errors exist.

### Commit Messages

This project follows [Conventional Commits](https://www.conventionalcommits.org/). Use these prefixes:

- `feat:` - New features → Features section in CHANGELOG
- `fix:` - Bug fixes → Bug Fixes section
- `docs:` - Documentation → Documentation section
- `refactor:` - Code refactoring → Refactoring section
- `test:` - Tests → Testing section
- `chore:` - Maintenance → Miscellaneous section

Examples:
```bash
feat(parser): add YAML bibliography support
fix: handle empty author fields gracefully
docs: update installation instructions
```

### Debugging
Use the `MDBOOK_LOG` environment variable to enable debug logging:
```bash
MDBOOK_LOG=mdbook_bib=debug mdbook build
MDBOOK_LOG=debug mdbook build  # Global debug for all targets
MDBOOK_LOG=mdbook_bib=debug,handlebars=warn mdbook build  # Module-specific levels
```

### Testing with Example Books
The repository includes example books for integration testing in `example_books/`:
- `basic/` - Custom backend with Handlebars templates and CSS
- `csl_ieee/`, `csl_chicago/`, `csl_nature/`, `csl_alphanumeric/` - CSL backend examples
- `pandoc/` - Pandoc-compatible citation syntax
- `manual/` - The documentation book (in root directory)

To test the plugin with an example book:
```bash
cd example_books/basic
mdbook build
```

### Release Process

**Prerequisite**: Install [git-cliff](https://git-cliff.org/) for changelog generation:
```bash
cargo install git-cliff
```

Use the Makefile for releases:
```bash
make release                  # Auto-increment patch version (0.5.1 → 0.5.2)
make release VERSION=x.y.z    # Specific version
make release DRY_RUN=1        # Preview without making changes
```

The release process:
1. **check-release** - Validates version format and clean working directory
2. **update-version** - Updates `Cargo.toml` and `.github/workflows/doc.yml`
3. **update-lockfile** - Regenerates `Cargo.lock`
4. **update-changelog** - Generates `CHANGELOG.md` using git-cliff
5. Commits all changes with message "Prepare for release vX.Y.Z"
6. Creates annotated tag `vX.Y.Z`
7. Pushes commit and tag atomically to origin

## Architecture

### Core Components

**src/lib.rs** - Main preprocessor logic
- `Bibliography` struct implements the `Preprocessor` trait
- Processes mdBook content to:
  - Load bibliography from BibLaTeX files or download from Zotero
  - Replace citation placeholders (`{{#cite key}}` or `@@key`) with formatted citations
  - Generate bibliography chapter with cited references
  - Optionally add bibliography sections at the end of each chapter

**src/config.rs** - Configuration handling
- Parses `[preprocessor.bib]` section from book.toml
- Supports configuration options:
  - `bibliography` - Path to .bib file
  - `zotero-uid` - Alternative to .bib file
  - `title` - Bibliography section title (default: "Bibliography")
  - `cited-only` - Show only cited refs (via `render-bib`)
  - `order` - Sort order: none/key/author/index
  - `add-bib-in-chapters` - Add bibliography to each chapter
  - `hb-tpl`, `cite-hb-tpl` - Custom Handlebars templates
  - `css`, `js` - Custom styling and scripts

**src/main.rs** - CLI entry point
- Uses `clap` for command-line argument parsing
- Initializes `tracing` logging via `MDBOOK_LOG` environment variable
- Handles `supports` subcommand for renderer compatibility

### Citation Processing Flow

1. **Bibliography Loading**: Load from .bib/.yaml file or download from Zotero API
2. **Parsing**: Use `hayagriva` crate to parse BibTeX/YAML into `BibItem` structures
3. **Citation Replacement**: Scan chapters for citation patterns using regex:
   - `{{#cite citation-key}}` - Handlebars-style
   - `@@citation-key` - Shorthand notation
4. **Indexing**: Track citation order and assign indices on first appearance
5. **Rendering**: Use Handlebars templates to format citations and bibliography
6. **Chapter Generation**: Create bibliography chapter with all cited references

### Template System

The plugin uses Handlebars for customizable rendering:
- **References template** (`src/render/references.hbs`) - Bibliography entries
- **Citation template** (`src/render/cite_key.hbs`) - Inline citations
- **CSS** (`src/render/satancisco.css`) - Default styling
- **JavaScript** (`src/render/copy2clipboard.js`) - Client-side functionality

Templates receive these data structures:
- `BibItem`: citation_key, title, authors, pub_month, pub_year, summary, url, index
- `Citation`: item (BibItem), path (relative path to bibliography page)

### Key Data Structures

**BibItem** - Represents a bibliography entry
- Fields: citation_key, title, authors (Vec<Vec<String>>), pub_month, pub_year, summary, url, index
- Serialized to JSON for Handlebars rendering

**IndexMap<String, BibItem>** - Main bibliography storage
- Preserves insertion order from .bib file
- Keyed by citation keys

## Testing Strategy

Tests are in `src/tests.rs` and use:
- `tempfile` crate for temporary test files
- `mdbook-driver` crate for integration testing with mdBook
- Test fixtures in `example_books/` directory

## GitHub Workflows

Workflows run sequentially with failure gates after a tag push:

1. **release.yml** - Builds binaries for Linux, Windows, macOS
   - If any build fails → stops (no publish)
2. **publish.yml** - Publishes to crates.io
   - Only runs if Release succeeded
   - If publish fails → stops (no docs)
3. **doc.yml** - Deploys documentation to GitHub Pages
   - Only runs if Publish succeeded

Additionally:
- **test.yml** - Runs on commits/PRs to master (format, clippy, build, test)

## Versioning

From version 0.5.0+, the minor version tracks mdBook's minor version for compatibility.
- Documentation about how to develop extensions to mdbook like this (mdbook-bib) and be found in the mdbook documentation in this web page: https://rust-lang.github.io/mdBook/for_developers/index.html and the links referenced in that page.