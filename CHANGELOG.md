# mdbook-bib Changelog

All notable changes to this project will be documented in this file.
## [0.5.2] - 2026-01-20

### Documentation

- Remove migration guide from manual (f66be10)


### Other

- Update Changelog for Version (487d635)

- Replace nom-bibtex with hayagriva parser

Integrate hayagriva 0.9 as the new bibliography parser, replacing nom-bibtex.
This provides better BibTeX/BibLaTeX parsing and lays the foundation for
YAML bibliography support and future CSL integration.

Key changes:
- Add hayagriva 0.9 dependency, remove nom-bibtex
- Implement new parser in src/parser/hayagriva_parser.rs (210 lines)
- Add BibFormat enum to support BibTeX and YAML formats
- Update io module to detect file format (.bib, .bibtex, .yaml, .yml)
- Simplify parser/mod.rs to just re-export hayagriva parser (4 lines, down from 219)
- Remove preprocessing hacks (quote removal, R@10 pattern replacement)
- Better author name parsing using hayagriva's Person objects

Benefits:
- More robust BibTeX parsing with better error messages
- No preprocessing required - hayagriva handles edge cases properly
- Support for both .bib and .yaml bibliography files
- Cleaner codebase: removed 215 lines of parser code
- Better handling of author names (resolves issue #44)
- Foundation for Phase 5 CSL integration

Test updates:
- Updated test BibTeX data to use proper format (numeric months)
- Removed internal date extraction test (now tested through integration)
- All 16 tests pass with hayagriva parser
- Test book builds successfully

All existing functionality preserved - zero breaking changes. (4c61e45)

- Expand BibItem with extended fields and enhanced templates

Add comprehensive bibliography field support by expanding BibItem structure
with 13 new optional fields extracted from hayagriva. Update Handlebars
templates to conditionally display extended metadata. Maintain 100%
backward compatibility with existing bibliographies.

Key changes:
- Expand BibItem with optional fields: entry_type, doi, pages, volume,
  issue, publisher, address, isbn, issn, editor, edition, note, organization
- Implement extraction functions for all hayagriva-supported fields
- Update references.hbs template with conditional sections for:
  * DOI with clickable hyperlinks
  * Publication details (volume, issue, pages)
  * Publisher and address information
  * ISBN/ISSN display
  * Editor information
  * Entry type badges
- Add 5 comprehensive tests covering extended fields, serialization,
  and backward compatibility
- Fix month indexing (hayagriva uses 0-indexed, converted to 1-indexed)
- Update DUMMY_BIB_SRC to use standard BibTeX month constants

All 22 tests passing. Backward compatible - existing entries work unchanged.

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com> (f8b2d2a)

- Implement BibliographyBackend trait abstraction

This commit implements the backend abstraction layer that separates
bibliography rendering logic from the core preprocessor, enabling
support for multiple rendering strategies (Legacy Handlebars and
future CSL support).

- Created `BibliographyBackend` trait in `src/backend/mod.rs`
- Defined `BackendMode` enum (Legacy, CSL)
- Defined `CitationContext` struct for citation rendering

- Implemented in `src/backend/legacy.rs`
- Preserves existing Handlebars-based rendering behavior
- Provides 100% backward compatibility
- Includes comprehensive unit tests

- Created stub implementation in `src/backend/csl.rs`
- Placeholder for Phase 5 CSL integration
- Returns simple placeholders with Phase 5 notice
- Includes basic tests

- Extended Config struct with `backend` and `csl_style` fields
- Added backend mode detection logic
- Defaults to Legacy mode for backward compatibility

- Updated `src/citation/mod.rs` to use backend trait
- Updated `src/renderer/mod.rs` to use backend trait
- Modified `src/lib.rs` to instantiate appropriate backend
- All tests updated to use LegacyBackend wrapper

- All 26 tests passing
- Code formatted with rustfmt
- No clippy warnings
- Backward compatibility verified

Phase 4 complete. Ready for Phase 5: CSL Integration. (3cc40d5)

- Full CSL Integration with hayagriva BibliographyDriver

This commit implements complete CSL (Citation Style Language) support,
allowing users to choose from 80+ bundled citation styles for standardized
academic formatting.

- Fully functional CslBackend using hayagriva's BibliographyDriver
- Support for 15+ common citation styles with friendly aliases:
  - IEEE, APA, Chicago (author-date & notes), MLA, Nature, Vancouver
  - Harvard, ACM, ACS, AMA, Springer, Cell, Elsevier variants
- Automatic style loading from hayagriva's bundled archive
- Proper error handling for invalid or dependent styles

- Added `hayagriva_entry: Option<Arc<hayagriva::Entry>>` field
- Parser now stores original hayagriva Entry for CSL rendering
- Maintains full backward compatibility with existing code

- Enabled `archive` feature in hayagriva dependency
- CSL backend now returns Result for proper error handling
- Updated lib.rs to handle CSL backend initialization errors

- Removed PartialEq from BibItem (Arc<Entry> cannot derive it)
- Removed PartialEq from Citation (depends on BibItem)
- Added Arc wrapper for Entry to enable cloning and sharing

- Primary: Uses ArchivedStyle::by_name() for convenience
- Fallback: Custom name mapping for common aliases
- Extracts IndependentStyle from Style enum
- Rejects dependent styles with clear error messages

- Creates BibliographyDriver for each citation
- Builds CitationRequest with style and locales
- Renders using hayagriva's CSL engine
- Returns formatted HTML with proper CSL styling

- Generates complete bibliography entries per CSL style
- Wraps output in <div class='csl-entry'> for styling
- Handles missing entries gracefully with fallback text

- 7 new CSL-specific tests covering:
  - Backend creation with multiple styles
  - Invalid style handling
  - Citation formatting (IEEE, Nature)
  - Reference formatting (APA)
- All 32 tests passing
- Code formatted with rustfmt
- Zero clippy warnings

- Legacy (Handlebars) mode unchanged and fully functional
- All existing tests pass without modification
- Default configuration uses Legacy mode
- CSL mode is opt-in via `backend = "csl"` or `csl-style` config

Phase 5 complete. Full hayagriva integration achieved.

Sources:
- [Hayagriva GitHub](https://github.com/typst/hayagriva)
- [Hayagriva Documentation](https://docs.rs/hayagriva/latest/hayagriva/)
- [Citation Style Language](https://citationstyles.org/)
- [CSL Styles Repository](https://github.com/citation-style-language/styles) (690ed4d)

-     Add CSL test books and fix ANSI code stripping

    - Created three demonstration test books showcasing CSL styles:
      * test_book_csl_ieee: IEEE numbered citations [1], [2]
      * test_book_csl_chicago: Chicago Author-Date (Author Year)
      * test_book_csl_nature: Nature superscript citations

    - Fixed ANSI escape code stripping in CslBackend:
      * Added strip_ansi_codes() helper function
      * Handles both ESC+[Nm sequences and bare [Nm codes
      * Applied to both format_citation() and format_reference()

    - Added comprehensive test coverage:
      * test_ansi_stripping_debug: Verifies raw output and stripping
      * test_format_citation_output_clean: End-to-end test

    All 34 tests passing. CSL backend fully functional with clean HTML output. (86bbe70)

- Fix CSL citation linking to bibliography entries

- Add markdown links from citations to bibliography anchors
- Handle different CSL formats:
  * Bracket styles (IEEE, Nature): [[1](url)]
  * Parenthetical styles (Chicago, APA): ([Author Year](url))
  * Superscript styles: [[num](url)]

- Strip outer delimiters from CSL output before wrapping in links
- Add anchor IDs to bibliography entries (id='citation_key')
- Add logging for citation replacement debugging

All three test books now have working citation->reference links:
- IEEE: [1] → clickable number
- Chicago: (Author Year) → clickable author-date
- Nature: 1 → clickable superscript number (6366ea9)

- Fix CSL numeric citation numbering (CRITICAL BUG FIX)

PROBLEM: All citations showed [1] regardless of which reference they cited.
This was because each citation created a new BibliographyDriver, so hayagriva
always assigned number 1.

SOLUTION:
1. For numeric styles (IEEE, Nature, Vancouver), use item.index directly
   - The index is already assigned by citation/mod.rs based on first appearance
   - No need to call hayagriva for citation formatting

2. For author-date styles (Chicago, APA), continue using hayagriva formatting
   - These need "(Author Year)" format from CSL

3. Bibliography rendering now prepends index for numeric styles

VERIFICATION:
- IEEE: Citations correctly numbered [1], [2], [3], [4], [5]
  - Same reference reuses same number across chapters
  - Bibliography shows [1], [2], [3], [4] with full references

- Nature: Citations [1], [2], [3] with Nature-style bibliography

- Chicago: (Author Year) format unchanged and working correctly

All 34 tests passing. Citations now have proper sequential numbering! (b875ae1)

- Update HAYAGRIVA_INTEGRATION_PLAN.md to reflect completed phases

- Simplified to two backend modes: Legacy (Handlebars) and CSL
- Removed tri-mode system (Pure CSL option removed)
- Marked Phases 1-5 as COMPLETED with actual achievements
- Phase 6 (Testing, Documentation, Polish) still TODO
- Updated architecture diagram to show dual-mode system
- Added backend comparison table
- Updated module structure to reflect actual implementation
- Added test books table and running instructions (4f932a0)

- Add documentation for dual backend system (Legacy and CSL)

- Update README.md with rendering backends section and quick start examples
- Update manual/src/intro.md with backend comparison table
- Create manual/src/legacy.md with complete Handlebars template reference
- Create manual/src/csl.md with CSL styles documentation
- Simplify manual/src/config.md with configuration reference table
- Update manual/src/SUMMARY.md with new page links
- Update HAYAGRIVA_INTEGRATION_PLAN.md to reflect Phase 6 documentation progress (0c1b7cd)

- Add migration guide for v0.5.x to v1.0.0 upgrade

- Create manual/src/migration.md with upgrade instructions
- Document backwards compatibility guarantees
- Add guidance on choosing Legacy vs CSL backend
- Include new template variables reference
- Add troubleshooting section
- Update intro.md with migration guide reference (cf3738a)

- Add comprehensive test suite for Phase 6

Tests added:
- Regression tests for Legacy backend output verification
- Backend-specific tests (Legacy vs CSL comparison)
  - Citation format differences
  - Numeric vs author-date styles
  - Superscript style (Nature)
  - Reference formatting
- YAML bibliography tests
  - Parsing, Legacy rendering, CSL rendering
  - YAML vs BibTeX equivalence
- Zotero integration tests (config parsing, URL construction)
- Per-chapter bibliography config tests
- Edge case tests:
  - Empty bibliography, malformed BibTeX
  - Citation to nonexistent key
  - Special characters, Unicode authors
- Integration tests for all CSL test books

Total: 58 tests passing (e4ea09a)

- Complete security audit

- Ran cargo audit: no vulnerabilities found (287 deps scanned)
- 1 warning: paste crate unmaintained (transitive, not a risk)
- Input validation covered by hayagriva parser tests
- HTML output sanitized for citation keys (8f230db)

- Fix per-chapter bibliography numbering and Nature reference format

Two bugs fixed:
1. Per-chapter bibliography showed all entries as [1] because
   add_bib_at_end_of_chapters was called before indices were assigned.
   Now expand_cite_references_in_book runs first to assign indices,
   and passes per-chapter citation sets to add_bib_at_end_of_chapters.

2. Nature-style references used "[1]" format instead of "1." format.
   Added is_superscript check in format_reference to use correct format. (5a3c04a)

- Rename Legacy backend to Custom backend

The "Legacy" name implied deprecation, which was misleading since
the Handlebars backend offers unique features (custom templates,
interactive elements, full layout control) that CSL doesn't support.

"Custom" better describes the value proposition: users can build
their own citation and bibliography styles.

Changes:
- Rename BackendMode::Legacy to BackendMode::Custom
- Rename LegacyBackend to CustomBackend
- Rename src/backend/legacy.rs to src/backend/custom.rs
- Rename manual/src/legacy.md to manual/src/custom.md
- Update config option from backend="legacy" to backend="custom"
- Update all documentation and tests (61f7127)

- Fix clippy warning and optimize regex compilation

- Use if let Some pattern instead of is_some() + unwrap() in tests
- Move ANSI regex to lazy_static for one-time compilation (65fa9e4)

- Improve error handling and fix documentation

- Replace .unwrap() calls in config.rs with proper error handling using
  helper functions that return descriptive errors for invalid TOML types
- Handle non-UTF-8 paths gracefully instead of panicking
- Log Zotero download errors before falling back to empty string
- Enhance CSL style error message with documentation link
- Fix migration docs: remove non-existent series field, add isbn/issn/organization (fa57261)

- Auto-update changelog!!! (12aa182)

- Enhance manual and test_book bibliography rendering and citation syntax

- Updated citation syntax in documentation to clarify usage:
  - Changed from `{{#cite key}}` and `@@key` to **`{{#cite key}}`** and **`@@key`** for better visibility.
- Improved bibliography entry structure in templates, transitioning from `<div>` to `<article>` for semantic clarity.
- Added CSS styles for modern bibliography presentation, integrating theme support.
- Introduced new configuration options in `book.toml` for citation templates and styles.
- Enhanced error handling in citation extraction and rendering processes.

This aims to close #66 (d3c915b)

- Refactor BibItem structure to use Option types for publication fields

- Updated `pub_month`, `pub_year`, and `summary` fields in `BibItem` to be of type `Option<String>` for better handling of optional data.
- Adjusted related parsing and extraction functions to accommodate the new types, ensuring compatibility with existing tests.
- Modified test cases to reflect changes in the expected data structure, ensuring robustness in bibliography entry handling. (92af803)

- Add citation placeholder replacement function for improved handling

- Introduced `replace_citation_placeholder` function to centralize citation replacement logic for both `{{#cite ...}}` and `@@cite` patterns.
- Enhanced error handling for missing or invalid citations, logging appropriate messages.
- Updated `replace_all_placeholders` to utilize the new function, streamlining the citation processing workflow.
- This refactor improves code maintainability and readability while ensuring consistent citation formatting. (70425c7)

- Enhance Makefile and documentation for improved release management

- Updated Makefile to include a dry-run mode for simulating commands without changes.
- Added a new target `show-version` to display current and next version information.
- Improved help output in Makefile to clarify available targets and options.
- Enhanced documentation in `dev.md` to detail the release process, including quick release commands and dry-run usage.
- Updated settings for local development to include new make commands for better integration. (2595a35)

- Auto-update changelog!!! (147095f)

- Perform test re-organization for improved clarity and maintainability

- Added a comprehensive testing section to `dev.md`, detailing how to run tests and organize them into logical modules.
- Expanded the documentation to include descriptions of test utilities and examples for better understanding.
- Streamlined the test suite in `src/tests.rs` by organizing tests into submodules for improved maintainability and readability.
- This update aims to facilitate easier onboarding for new contributors and enhance the overall development experience. (b7186c3)

- Enhance test organization with rstest for parametrization

- Added new dependencies including `futures-macro`, `futures-timer`, `glob`, `proc-macro-crate`, `relative-path`, `rstest`, `rustc_version`, `semver`, and `toml_edit` to `Cargo.lock` and `Cargo.toml`.
- Refactored tests in `src/tests/backend.rs`, `src/tests/citation.rs`, and `src/tests/parser.rs` to utilize the `rstest` crate for parameterized testing, improving test clarity and maintainability.
- Expanded documentation in `manual/src/dev.md` to include examples of using `rstest` for parameterized tests, enhancing developer onboarding and understanding of testing practices. (4767776)

- Auto-update changelog!!! (2046c8b)

- Refactor template loading in config.rs for improved maintainability

- Introduced a new `load_template` helper function to streamline the loading of various templates (HB, CSS, JS) from specified paths or defaults.
- Replaced repetitive code blocks with calls to `load_template`, enhancing code clarity and reducing duplication.
- Updated logging to provide clearer information on which templates are being used. (fb3feeb)

- Auto-update changelog!!! (2ad4959)

- Enhance CSL backend with new citation styles and improved style loading

- Added support for new citation styles: `vancouver-superscript`, `elsevier-vancouver`, `alphanumeric`, `mla8`, and `springer-basic-author-date`.
- Refactored style loading logic to utilize a registry for common aliases, improving the resolution of style names.
- Updated documentation in `csl.md` to reflect the new styles and their formats.
- Enhanced caching of style information for quicker access to citation format flags. (04b9a05)

- Enhance CSL backend with style resolution improvements and documentation updates

- Introduced a new style resolution strategy that includes registry aliases and fallback detection for citation formats.
- Added runtime detection of citation characteristics for styles not in the registry, improving citation formatting accuracy.
- Updated `csl.md` documentation to reflect the new style resolution methods and fallback limitations.
- Refactored `CslBackend` to utilize detected formats and improve clarity in citation style handling.
- Enhanced tests to verify fallback style format detection and registry style characteristics. (9206b23)

- Add new tests and documentation updates

- Introduced support for the `vancouver-superscript` and `alphanumeric` citation styles, enhancing the CSL backend's capabilities.
- Updated `csl.md` documentation to explain the differences between Vancouver styles and the use of alphanumeric citations.
- Added comprehensive tests for the new citation styles to ensure correct rendering and functionality.
- Enhanced the style registry with functions for retrieving detailed style information and formatting style lists for better usability. (6298dc8)

- Auto-update changelog!!! (32368b4)

- Enhance CSL backend with full support for alphanumeric citation style

- Updated the `CslBackend` to handle alphanumeric citations using author-based labels instead of sequential numbers.
- Introduced new methods to check for label styles and format fallback bibliography entries.
- Enhanced documentation in `csl.md` to clarify the alphanumeric style's label format and usage.
- Added tests to verify correct rendering of alphanumeric citations and references, ensuring accurate functionality. (03525c5)

- Refactor CSL backend to unify citation format handling

- Introduced a comprehensive `CitationFormat` struct to encapsulate content type and rendering style, replacing individual boolean checks for numeric, label, and superscript styles.
- Updated `CslBackend` methods to utilize the new citation format structure, enhancing clarity and maintainability.
- Refactored style registry to support the new format, ensuring accurate detection and rendering of citation styles.
- Enhanced documentation to reflect changes in citation format handling and updated tests to verify correct functionality across various citation styles. (4d0aabb)

- Enhance CSL backend citation text processing and documentation

- Updated the `CslBackend` to improve citation text handling by stripping leading/trailing brackets and parentheses from citation text before rendering.
- Added a new section in `csl.md` to document the behavior of citation text processing, including potential edge cases with citation keys.
- Enhanced logging to warn when no citation is returned, ensuring better traceability of citation handling issues. (9f2d9a1)

- Auto-update changelog!!! (ee410c4)

- Add TL;DR to README.md (4e673ea)

- Enhance citation handling with Pandoc compatibility

- Enabled through new configuration options
- Added support for Pandoc citation syntax, allowing recognition of `@key`, `[@key]`, `[-@key]` alongside existing formats.
- Introduced a new `CitationSyntax` enum to manage citation patterns in the configuration.
- Updated the `replace_all_placeholders` function to process Pandoc-style citations, ensuring proper rendering based on the selected syntax.
- Enhanced the `CitationContext` to include citation variant information for more flexible rendering.
- Updated documentation and tests to reflect the new citation syntax capabilities and ensure comprehensive coverage.

This aims to solve #51 (61cc61c)

- Refactor citation handling and remove unused dependencies

- Removed `fancy-regex` dependency in favor of the standard `regex` crate for citation pattern matching.
- Updated citation processing logic to utilize structured author data for improved formatting in inline citations.
- Enhanced documentation in `csl.md` to clarify citation variants and their effects on different style types.
- Added tests to ensure proper functionality of the updated citation patterns and handling of edge cases. (0795e34)

- Update citation key handling and documentation for compatibility

- Expanded the documentation in `config.md` to clarify citation key formats for both native and Pandoc patterns, including rules for keys starting with digits.
- Enhanced regex patterns in `mod.rs` to reflect the differences in key formats between mdbook-bib and Pandoc, ensuring backward compatibility and adherence to specifications.
- Added notes on cross-tool compatibility and usage recommendations for digit-prefixed keys in Pandoc syntax. (c9319e5)

- Update config.md to document unsupported Pandoc features for citations (e6ad4f8)

- Enhance author formatting in CSL backend and add comprehensive tests

- Updated the `format_authors_for_citation` method to return "Unknown" for entries without authors or with only empty names, improving robustness.
- Added filtering for empty name parts to ensure accurate author representation.
- Introduced multiple unit tests to validate the behavior of author formatting under various scenarios, including single, multiple, and empty authors. (8c1bcd4)

- Auto-update changelog!!! (68b3cb7)

- Remove HAYAGRIVA_INTEGRATION_PLAN.md and update documentation for clarity and consistency

- Deleted the HAYAGRIVA_INTEGRATION_PLAN.md file as it is no longer needed.
- Revised README.md to enhance feature descriptions and improve clarity.
- Updated manual documentation across multiple files, including config.md, csl.md, and custom.md, to reflect changes in citation handling and backend options.
- Added a new FUNDING.yml file to support funding options for the project. (d01f48b)

- Auto-update changelog!!! (61d4743)

- Enhance workflows and documentation for release process

- Updated CLAUDE.md to include guidelines for commit messages following Conventional Commits, improving clarity for contributors.
- Revised Makefile to add new targets for `check-release`, `update-lockfile`, and `update-changelog`, streamlining the release process.
- Modified GitHub workflows to ensure sequential execution and proper dependency handling, including updates to `publish.yml`, `release.yml`, and `test.yml`.
- Adjusted paths in README.md and manual documentation for consistency and clarity regarding example bibliography files.
- Improved error handling and validation steps in the release process to prevent issues during automated deployments. (6d10b16)


## [0.5.1] - 2026-01-14

### Other

- Auto-update changelog!!! (d77ac6f)

- Update Changelog for Version (1fc153c)

- Add Claude Code config (2ba1b4c)

- "Claude PR Assistant workflow" (e359f5f)

- "Claude Code Review workflow" (2f2e708)

- Auto-update changelog!!! (a965f6d)

- Fix override of mdBook's default built-in {{#...}} expressions

This aims to close #52 (ef218ae)

- Refactor regex pattern in tests to use constant REF_PATTERN and improve citation key validation (f691a71)

- Auto-update changelog!!! (7af8967)

- Fix render of @@ citations with a dot (.) at the end

This aims to close #49 (7808e8d)

- Add DOI followed by period (@@10.1145/3508461.) test case (4a9b842)

- Add new test coverage cases for test_at_ref_followed_by_punctuation (54ec7bc)

- Update both citation patterns to be BibLaTeX-compliant and homogeneous (09422dd)

- Auto-update changelog!!! (422c49e)

- Update workflow branch filter from v0.0.* to v0.5.* (61f8d97)

- Add github_dispatch to manually trigger doc release (25b5bb2)

- Single handlebar registry for storing the plugin templates

Before a new instance was created each time the templates were gonna be used.

This aims to close #47 (b82acc9)

- Address some minor issues:1. Error handling using the ? operator and return early with a meaningful error message
2. Remove Unnecessary Clone
3. Remove Code Duplication in tests (5d69e24)

- Auto-update changelog!!! (83d0747)

- Extract modules for improved architecture (b48e83a)

- Auto-update changelog!!! (fbf4e98)

- Prepare for release v0.5.1 (6296a86)


## [0.5.0] - 2025-12-06

### Bug Fixes

- Fix spelling error and remove depreciated "Multilingual" (d44a41e)

- Fix preprocessor selector issue (72e92d0)


### Other

- Update Changelog for Version (1721351)

- Add option for adding the bib cites of the chapter  at the end of each page

This aims to fulfill the feature described in #38 (616b32e)

- Fix  option for bib cites of each chapter

Enhance citation extraction with regex patterns

This update introduces regex-based matching for citation keys in the  function, improving the handling of both regular cites and at_at placeholders. The use of  allows for efficient regex compilation, enhancing performance and clarity in citation processing.

This aims to fulfill the feature described in #38 (092256b)

- Fix version in doc github actions flow (cfba8d6)

- Add Makefile to automate the release process

make release VERSION=0.0.8 (2522834)

- Add explicit logging for nom_bibtex attribute parsing

Added improved error handling and logging in the bibliography parsing function to provide clearer insights during processing.

Also, enhance documentation with debugging instructions

This addressed #50 (c5aea1e)

- Auto-update changelog!!! (a77fae7)

- Update sources for build (86b45b6)

- Remove unused dependency (baa98b1)

- Update Cargo.lock (2ae4b8b)

- Fix minor glitches in manual after @tompkins-ct mdbook version upgrade to v0.5.1 (d540241)

- Add explicit checks for the presence of the [preprocessor.bib] section

Added logging for missing sections and errors during configuration reading, ensuring smoother processing and clearer insights into potential issues. (231b826)

- Auto-update changelog!!! (e675011)

- Replace log with tracing

this closes #57 (e2d494a)

- Update logging-related documentationThis is related to #57 (2a9ff2e)

- Auto-update changelog!!! (95736dd)

- Prepare for release v0.5.0 (f4f5590)


## [0.0.7] - 2025-07-24

### Other

- Update Changelog for Version (f6e01e7)

- Remove duplicate code for finding cite placeholders in text

Aims to close #46 (bb74667)

- Auto-update changelog!!! (7566092)

- Update Cargo.lock version and refactor code for improved clarity

- Bump Cargo.lock version from 3 to 4.
- Refactor author retrieval in Bibiography to use `.first()` instead of `.get(0)`.
- Simplify HashMap creation in build_bibliography by using `.map()` for cloning keys and values.
- Modify test to check for placeholders more efficiently. (01f65ab)

- Bump nom-bibtex to 0.5.0 and Update dependencies in Cargo.lock and Cargo.toml

- Remove outdated package entries for `arrayvec`, `bytecount`, and `lexical-core` in Cargo.lock.
- Add new package `minimal-lexical` in Cargo.lock.
- Bump versions for `nom`, `nom-bibtex`, `nom-tracable`, and `nom_locate` in Cargo.lock and Cargo.toml.
- Update `nom-bibtex` version in Cargo.toml from 0.3.0 to 0.5.0. (e584e47)

- Update Cargo.toml and remove old reference to old builder (bd870ac)

- Fix this error

596 | fn find_placeholders(contents: &str) -> Vec<Placeholder> {
    |                                ^^^^         ----------- the same lifetime is hidden here
    |                                |
    |                                the lifetime is elided here (3c3ec21)

- Auto-update changelog!!! (950e3c3)

- Refactor citation placeholder handling and improve regex matching

    This solves the stack overflow problem that @man-chi had in #39 when trying to build mdbook with an @@DUMMY:1 citation.

    - Introduced `RefCell` for interior mutability in `replace_all_placeholders` to manage state more effectively.
    - Replaced the previous placeholder finding logic with regex-based matching for improved performance and clarity.
    - Updated tests to validate the new citation handling and regex patterns.
    - Added new bibliography entries for testing purposes. (10b6aaa)

- Enhance citation handling and regex patterns

This solves the problem reported by @hfiguiere
 in #39

- Updated `AT_REF_PATTERN` regex to allow citation keys with dots.
- Added a new test for citation replacement to ensure proper handling of keys with dots.
- Improved documentation in `intro.md` to clarify references and issues related to citations.
- Added a new bibliography entry for testing purposes. (3609586)

- Auto-update changelog!!! (b46cff6)

- Prepare for release v0.0.7 (6525ec0)


## [0.0.6] - 2023-09-02

### Bug Fixes

- Handle author names which include 'and' (b5e0441)


### Other

- Update Changelog for Version (34f3588)

- Update lockfile to get past proc-macro2 issue

had to also update some other files to get around new issues raised by
newer clippy. (72cb18b)

- Auto-update changelog!!! (f873588)

- Test the bug reported/fixed in #44/#45 in a real book (c947c50)

- Fix problem with toml version

Some problems arised when two toml versions were conflicting from rusty-hook and mdbook etc. (dfe7da6)

- Prepare for release v0.0.6 (601ca95)


## [0.0.5] - 2023-04-13

### Bug Fixes

- Fix citation template newlines, make author date style template (9e59ce4)

- Fix new inline formatting standard on nightly (edc824f)


### Documentation

- Document the new features (0a3629d)


### Other

- Update dependencies to get rid of errors and warnings, fix clippy warnings (7aea39b)

- Implement templating for inline citations (incomplete) (09bad99)

- Store authors as vector (e462147)

- Put templates in a separate folder for users to download (4b5886f)

- Add different reference sort order options, add index-style citation templates (0123a5c)

- Upgrade to Rust edition 2021, update all dependencies (edd2f68)

- Use ampersand in author/year citation (b840189)

- Provide full bib output file path to citation templates (5e41262)

- Auto-update changelog!!! (eb10f42)

- Prepare for release v0.0.5 (d59bc62)

- Auto-update changelog!!! (dc64226)

- Update Changelog for Version (a3114b1)

- Trigger Release workflow run Publish on v.0.0.x branches (ef1c45a)

- Trigger Release workflow at the same time that Crates Publishing (515fa89)

- Auto-update changelog!!! (2f82e91)

- Fix doc workflow (2cb541a)

- Auto-update changelog!!! (868bc7e)


## [0.0.4] - 2021-09-04

### Other

- Update Changelog for Version (a3101e1)

- Fix clippy errors with bool asserts and needless borrow (69b5b38)

- Release v0.0.3 binaries

As per @mlange request in #23 (a8fa0be)

- Restore the trigger for releases after publishing v0.0.3 binaries (0ecb4de)

- Add doc manual and publish to gh-pages (b66086e)

- Shorten README.md & reference the book for additional info

This closes #20 (c2e9407)

- Add bib title config param

Related to #6 (17518f0)

- Fix parsing of @ symbol in bibtex content

Some entries were throwing an error when the @ symbol was found.
The @ has been substituted for _at_

This will close #25. (ec6b81a)

- Allow custom reference styles

This aims to close #15 (8a4684e)

- Fix location of .bib in book scaffold and fix #24

This aims to fix and close #24 (496392c)

- Allow also citations with @ (91e181c)

- Link to bibliography from citations in subfolders

Citations in files lower than the book source root were
producing an incorrect link because they weren't
considering the relative path to the root.

This commits fixes it. (cca2ff7)

- Add rusty-hook integration (0dd92df)

- Run tests on Github PRs (19ca4d6)

- Prepare for release v0.0.4 (dd7e979)

- Disable dirty check for changelog prerelease (f6a35cb)

- Checkout the correct branch (28ca3a0)

- Checkout the correct branch in the git-auto-commit-action

Also: Remove the skip dirty option as seemed not to be necessary (5281693)

- Auto-update changelog!!! (3cfbaf2)

- Update Changelog for Version (6a74f11)

- Release v0.0.4 binaries (05a14c4)


## [0.0.3] - 2021-04-04

### Bug Fixes

- Fix all clippy warnings (0276070)


### Other

- Save zotero downloaded lib. Also code restructuring (f00e252)

- Add download badge and minimal doc improves (d18dd7d)

- Add rustfmt and clippy to CI (258b13e)

- Moved tempfile to dev-dependencies (91f5708)

- Collect cited references, provide an option to only list cited in refs list (79d7950)

- Add test for rendering of all/cited references, made test bib a raw string (311c228)

- Allow to copy citations to clipboard from Bibliography

A button is shown in every bib entry to copy the citation key to the clipboard.
Also removes the popup citation key on hover mentioned in #2.

This should close #6 and #2 (dfd2baa)

- Fix format (5807d14)

- Add back citation key to html rendering to jump from citations

Also:

-Add new line at the end of references.hbs
-Rename js function from oldCopyTextToClipboard to defaulCopyTextToClipboard (253f80b)

- Parse toml config table into struct (51e28a1)

- Warn and skip processing on failed config parsing instead of bubble up the error (217a162)

- Add section on configuration to README, minror text tweaks (a226b92)

- Remove unused toml option `renderer` (adf34a9)

- Add workflow to publish binaries to releases; more flexible tag pattern (9ccab98)

- [#12] Add test to check citations are replaced properly in book text (dff3d74)

- [#12] Add test to check only citation placeholders are returned (d72cda3)

- [#12] Add test for checking config attributes (7dafed3)

- [#12] Fix clippy error exposed in github (a8bf7d4)

- Parse fields `year` and `month` if `date` is not present (f9668cd)

- [#16] Consolidate config style with hyphens

Also, sorten the zotero user id name attribute in config

This aims to close #16 (12eb2a2)

- Fix README.md title

As per @mlange-42 comment in #15.

Also fix typo. (c67a4fe)

- Segregate extract_date function and add missing test

Complements #12 and #5 which are already closed (f1b90d3)

- Add release wf dependency  on publish wf  and link them to master branch (534c4e7)

- [#8] Extract url from bib entries and render a link on the title of the article

This aims to close #8 (49a75f4)

- Fix publish workflow to not to be triggered when pushing to master (56606d7)

- Add changelog prerelease workflow with github_changelog_generator (23638bf)

- Add changelog as part of publish workflow with github_changelog_generator (0bcec28)

- Execute changelog prerelease only if test workflow succeeds (2c7d810)

- Bump version to 0.0.3 manually (260526e)

- Fix problem with lexical-core-0.7.4

Problem arises with lexical-core-0.7.4 when trying to publish crate through github.

Solution:

- Bump version explicilty in Cargo.toml to use 0.7.5 (lexical-core dep comes transitively from nom, which in turn comes from nom-bibtex)
- More info: https://github.com/rust-lang/rust/issues/81654 (79b020c)


## [0.0.2] - 2021-02-14

### Other

- Fix repository in Cargo.toml (092345c)

- Improve README.md (74b5664)

- Add Zotero downloads for public libraries as BibLaTex format

NOTE: BibLaTex format is preferred as the nom-bibtex parser has more
problems with the old Bibtex (7ae9b0c)


## [0.0.1] - 2021-02-10

### Other

- First version working

Tasks:

- Add gitignore file
- Filter idea files
- Add basic tests, clean and reorganize code
- Organize imports and minor changes
- Format code a la rust
- Fix biblio content addition and date format in hbs
- Add MPL 2.0 license
- Add github workflows for tests based on actions-rs (a23a64e)

- Add install instructions and improve README.md (6a4a1af)

- Add publish github workflow. Improve README.md (55aa8fb)

- Shorten number of tags to avoid error at publishing crate

Error:
error: api errors (status 200 OK): invalid upload request: invalid length 8, expected at most 5 keywords per crate at line 1 column 3162 (079313a)

- Add missing description field in Cargo.toml. Required to publish crates (fcb3fdc)

- Add non-wildcard versions of deps to publish in crates (fb1b66e)



