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
| `integration.rs` | Full book builds (example_books variants) |
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

This project follows the [Conventional Commits](https://www.conventionalcommits.org/) specification for commit messages. This enables automatic changelog generation with proper categorization.

### Commit Format

```
<type>[optional scope]: <description>

[optional body]

[optional footer(s)]
```

### Common Types

| Type | Description | Changelog Section |
|------|-------------|-------------------|
| `feat` | New feature | Features |
| `fix` | Bug fix | Bug Fixes |
| `docs` | Documentation only | Documentation |
| `refactor` | Code change (no new feature or fix) | Refactoring |
| `test` | Adding/updating tests | Testing |
| `perf` | Performance improvement | Performance |
| `chore` | Maintenance tasks | Miscellaneous |
| `ci` | CI/CD changes | CI/CD |

### Examples

```sh
feat(parser): add YAML bibliography support
fix: handle empty author fields gracefully
docs: update installation instructions
refactor(render): extract template loading into separate module
test: add integration tests for CSL backend
```

### Before Committing

Format and lint your code:

```sh
cargo fmt --              # Format all code
cargo clippy --fix        # Run clippy with auto-fixes
cargo clippy --fix --tests  # Run clippy for tests
```

Pre-commit hooks (`.rusty-hook.toml`) block commits with formatting errors. GitHub's `test.yml` workflow runs on pushes to `master`. Look for problems in the [GitHub Actions](https://github.com/francisco-perez-sorrosal/mdbook-bib/actions).

## Versioning

From version 0.5.0, mdbook-bib's minor version tracks mdbook's minor version for compatibility.

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

### Prerequisites

Install [git-cliff](https://git-cliff.org/) for changelog generation:

```sh
cargo install git-cliff
```

### Available Targets

| Target | Description |
|--------|-------------|
| `release` | Complete release (update, commit, tag, push) |
| `update-version` | Update version in Cargo.toml and doc.yml |
| `update-cargo` | Update version only in Cargo.toml |
| `update-doc` | Update version only in doc.yml |
| `update-lockfile` | Regenerate Cargo.lock after version update |
| `update-changelog` | Generate CHANGELOG.md using git-cliff |
| `check-release` | Validate release preconditions |
| `show-version` | Show current and next version |

### Release Steps

The release process performs these steps locally:

1. **check-release** - Validate version format and clean working directory
2. **update-version** - Update `Cargo.toml` and `.github/workflows/doc.yml`
3. **update-lockfile** - Regenerate `Cargo.lock`
4. **update-changelog** - Generate `CHANGELOG.md` using git-cliff
5. Commit all changes with message `Prepare for release vX.Y.Z`
6. Create annotated tag `vX.Y.Z`
7. Push commit and tag atomically to origin

### GitHub Workflows (Sequential)

After the push, GitHub Actions run in sequence with failure gates:

1. **release.yml** - Builds binaries for Linux, Windows, and macOS
   - Uploads to [GitHub Releases](https://github.com/francisco-perez-sorrosal/mdbook-bib/releases)
   - If any platform fails → stops here (no publish)

2. **publish.yml** - Publishes to [crates.io](https://crates.io/crates/mdbook-bib)
   - Only runs if all Release builds succeeded
   - If publish fails → stops here (no docs)

3. **doc.yml** - Deploys documentation to [GitHub Pages](https://francisco-perez-sorrosal.github.io/mdbook-bib/)
   - Only runs if Publish succeeded

This ensures you never publish a crate that fails to build on any platform.

