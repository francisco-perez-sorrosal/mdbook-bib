# Dev

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

The release process can be triggered with the `make` command `make release VERSION=0.0.7` and it's composed by:

- Update new version (e.g. 0.0.7) in `Cargo.toml`, and `doc.yml` in the github workflows
- Do a commit with those changes with a message like `Prepare for release v0.0.7`
- Pull the remote changes to get the updated CHANGELOG.md by the previous commit from github: `git pull origin master`
- The release will be triggered by:
  - Creating a new tag in github: `git tag -a v0.0.7 -m "Version v0.0.7"`
  - Pushing the tag to github: `git push origin v0.0.7`
- The release will exercise the github workflows:
  - `publish.yml` - Publish the release
  - `releaese.yml` - Create the binary packages to release in [here](https://github.com/francisco-perez-sorrosal/mdbook-bib/releases)
  - `doc.yml`  - Will publish the book with this instructions [here](https://francisco-perez-sorrosal.github.io/mdbook-bib/)

## ToDo
Improve the process above when bored or when you want to improve friction points (e.g. the Changelog is updated post release, etc.)
