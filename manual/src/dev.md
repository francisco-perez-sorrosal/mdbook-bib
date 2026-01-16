# Dev

## Testing

### Running Tests

```sh
cargo test              # Run all tests
cargo test --workspace  # Run tests for the entire workspace
```

### Test Organization

Tests are organized into logical modules under `src/tests/`:

| Module | Purpose |
|--------|---------|
| `common.rs` | Shared fixtures, helpers, and `BibItemBuilder` |
| `parser.rs` | BibTeX/YAML parsing, date extraction, extended fields |
| `citation.rs` | Citation placeholder replacement, regex patterns |
| `config.rs` | Configuration parsing, Zotero, per-chapter settings |
| `backend.rs` | Custom and CSL backend formatting, regression tests |
| `integration.rs` | Full book builds (test_book, CSL style variants) |
| `edge_cases.rs` | Error handling, malformed input, unicode support |

### Test Utilities

The `common` module provides reusable test infrastructure:

- **`BibItemBuilder`** - Fluent builder for creating test `BibItem` instances
- **`dummy_bibliography()` / `yaml_bibliography()`** - Pre-parsed bibliographies (lazy-loaded)
- **`create_*_backend()` functions** - Factory functions for backend instances
- **Test fixtures** - `DUMMY_BIB_SRC`, sample text with citations

Example using the builder:

```rust
let item = BibItemBuilder::article("smith2024")
    .title("Test Article")
    .authors(&["Smith, John"])
    .year("2024")
    .build();
```

### Parametrized Tests

Tests use the `rstest` crate for parametrization. Example:

```rust
#[rstest]
#[case::ieee("ieee")]
#[case::apa("apa")]
#[case::chicago("chicago-author-date")]
fn test_csl_style(#[case] style: &str) {
    let backend = CslBackend::new(style.to_string()).unwrap();
    // test logic...
}
```

Each case runs as a separate test, making it easy to identify failures.

## Debug

The preprocessor uses the `tracing` library for logging. To enable debug output, use the `MDBOOK_LOG` environment variable:

```sh
MDBOOK_LOG=mdbook_bib=debug mdbook build
```

### Examples

```sh
MDBOOK_LOG=debug mdbook build # **Set globally for all targets**
```

```sh
MDBOOK_LOG=mdbook_bib=debug,handlebars=warn mdbook build # Debug specific modules like ours (mdbook_bib)
```

### Tips

- The default log level is `info` if `MDBOOK_LOG` is not set
- Noisy dependencies (`handlebars`, `html5ever`) are automatically silenced to `warn` unless explicitly specified
- Module paths (targets) are only displayed when `MDBOOK_LOG` is set

## Commits

Before submitting the commit, format code with:

```sh
# Format all code in the project
cargo fmt --
# Run clippy with auto-fixes
cargo clippy --fix
# Run clippy with auto-fixes for tests
cargo clippy --fix --tests
```

Commits will exercise the pre-commit hook in `.rusty-hook.toml` and will prevent the commit if formating errors are found

When committing to the `master` branch, the github workflow `test.yml` will be exercised. Look for problems in the [github actions](https://github.com/francisco-perez-sorrosal/mdbook-bib/actions).

After successful test pass, the `CHANGELOG.md` is updated.

## Versioning

**From version 0.5.0, moving the minor version of mdbook-bib to match the minor version of the mdbook project.**

## Release

The release process is managed via `make`. Run `make help` to see all options:

```sh
make help
```

### Quick Release

To release the next patch version (auto-incremented):

```sh
make release              # Releases 0.5.1 → 0.5.2 automatically
```

To release a specific version:

```sh
make release VERSION=1.0.0
```

### Dry-Run Mode

Add `DRY_RUN=1` to any command to simulate without making changes:

```sh
make release DRY_RUN=1              # Simulate with auto-version
make release DRY_RUN=1 VERSION=1.0.0  # Simulate specific version
make update-cargo DRY_RUN=1         # Simulate just Cargo.toml update
```

### Available Targets

| Target | Description |
|--------|-------------|
| `release` | Complete release (update, commit, tag, push) |
| `update-version` | Update version in Cargo.toml and doc.yml |
| `update-cargo` | Update version only in Cargo.toml |
| `update-doc` | Update version only in doc.yml |
| `show-version` | Show current and next version |

### Release Steps

The release process performs these steps:

1. Update version in `Cargo.toml` and `.github/workflows/doc.yml`
2. Commit changes with message `Prepare for release vX.Y.Z`
3. Create annotated tag `vX.Y.Z`
4. Push commit and tag to origin

After pushing, pull remote changes to get the updated CHANGELOG.md:

```sh
git pull origin master
```

### GitHub Workflows Triggered

The tag push triggers these workflows:

- `publish.yml` - Publishes to crates.io
- `release.yml` - Creates binary packages in [Releases](https://github.com/francisco-perez-sorrosal/mdbook-bib/releases)
- `doc.yml` - Publishes documentation to [GitHub Pages](https://francisco-perez-sorrosal.github.io/mdbook-bib/)

## ToDo

Improve the process above when bored or when you want to improve friction points (e.g. the Changelog is updated post release, etc.)
