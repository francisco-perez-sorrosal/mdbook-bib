# Hayagriva Integration Plan for mdbook-bib

## Executive Summary

Comprehensive integration of hayagriva into mdbook-bib to add robust bibliography parsing, CSL citation styles, and expanded field support. This plan includes full architectural refactoring for long-term maintainability.

**Key Features**: Two backend modes - Legacy (Handlebars) for full customization, and CSL for standard academic citation styles.

**Release Target**: v1.0.0

## Design Philosophy

### Core Principles

1. **Dual Backend System**: Legacy (Handlebars) for customization, CSL for standard formats
2. **Best of Both Worlds**: Full template control OR standardized academic formats
3. **100% Backwards Compatible**: All existing books work without config changes
4. **Clean Architecture**: Proper abstractions, traits, and module separation
5. **Test-Driven**: Maintain test coverage throughout refactoring

### Backend Comparison

| Feature | Legacy (Handlebars) | CSL Backend |
|---------|---------------------|-------------|
| Citation formatting | Custom templates | IEEE, Chicago, Nature, APA, etc. |
| Bibliography layout | Custom templates | Standard CSL formatting |
| Interactive elements | ✅ Full control | Basic (links only) |
| Copy buttons | ✅ Via templates | ❌ |
| Collapsible details | ✅ Via templates | ❌ |
| Custom layouts | ✅ Full control | ❌ |
| Configuration | More complex | Simple |
| Use case | Power users | Standard academic |

## Architecture Overview

### Dual-Mode System

```text
                    ┌─────────────────────┐
                    │  Bibliography       │
                    │  Preprocessor       │
                    └──────────┬──────────┘
                               │
                    ┌──────────▼──────────┐
                    │  Config Loading     │
                    │  (book.toml)        │
                    └──────────┬──────────┘
                               │
                ┌──────────────▼──────────────┐
                │  Detect Backend Mode        │
                └──────────────┬──────────────┘
                               │
              ┌────────────────┴────────────────┐
              │                                 │
    ┌─────────▼──────────┐           ┌─────────▼──────────┐
    │ Legacy Backend     │           │ CSL Backend        │
    │ (Handlebars)       │           │ (hayagriva)        │
    │                    │           │                    │
    │ Full template      │           │ Standard academic  │
    │ customization      │           │ citation styles    │
    └─────────┬──────────┘           └─────────┬──────────┘
              │                                 │
              └────────────────┬────────────────┘
                               │
                    ┌──────────▼──────────┐
                    │  Rendered Output    │
                    │  (HTML with         │
                    │   citations)        │
                    └─────────────────────┘
```

### Module Structure (Implemented)

```text
src/
├── lib.rs                          # Preprocessor trait impl, orchestration
├── config.rs                       # Config struct and loading logic
├── parser/
│   ├── mod.rs                      # Parser module
│   └── hayagriva_parser.rs         # Hayagriva-based parser
├── backend/
│   ├── mod.rs                      # BibliographyBackend trait + BackendMode enum
│   ├── legacy.rs                   # LegacyBackend (Handlebars templates)
│   └── csl.rs                      # CslBackend (hayagriva CSL)
├── models/
│   └── mod.rs                      # BibItem, Citation structs
├── citation/
│   └── mod.rs                      # Citation replacement logic
├── renderer/
│   └── mod.rs                      # Bibliography HTML generation
├── render/                         # Default templates
│   ├── references.hbs              # Default bibliography template
│   ├── cite_key.hbs                # Default citation template
│   ├── satancisco.css              # Default CSS
│   └── copy2clipboard.js           # Copy functionality
├── io/
│   └── mod.rs                      # File loading utilities
├── file_utils.rs                   # Path utilities
├── main.rs                         # CLI entry point
└── tests.rs                        # Test suite
```

## Configuration

### book.toml Structure

```toml
[preprocessor.bib]
# === Source Options ===
bibliography = "bibliography.bib"       # Path to .bib or .yaml file
zotero-uid = "123456"                   # Alternative: Zotero user ID

# === Rendering Options ===
title = "Bibliography"                  # Bibliography section title
render-bib = "cited"                    # "cited" or "all"
order = "none"                          # "none", "key", "author", "index"
add-bib-in-chapters = true              # Add bibliography at end of each chapter

# === Backend Selection ===

# --- Option 1: Legacy (Handlebars) - Default ---
# Full customization via templates
backend = "legacy"                      # Optional, this is the default
hb-tpl = "render/my_references.hbs"     # Custom references template
cite-hb-tpl = "render/my_citation.hbs"  # Custom citation template
css = "render/my_style.css"
js = "render/my_script.js"

# --- Option 2: CSL Backend ---
# Standard academic citation styles
backend = "csl"
csl-style = "ieee"                      # ieee, chicago-author-date, nature, apa, etc.
```

### Backend Mode Detection

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackendMode {
    Legacy,  // Handlebars templates (default)
    Csl,     // CSL citation styles via hayagriva
}
```

## Implementation Phases

---

### Phase 1: Architectural Refactoring ✅ COMPLETED

**Goal**: Restructure codebase for clean separation of concerns.

**Achievements**:

- ✅ Created modular structure: `parser/`, `backend/`, `models/`, `citation/`, `renderer/`, `io/`
- ✅ Extracted parsing logic to `src/parser/hayagriva_parser.rs`
- ✅ Extracted citation replacement to `src/citation/mod.rs`
- ✅ Moved `BibItem` and `Citation` to `src/models/mod.rs`
- ✅ Created `BibliographyBackend` trait in `src/backend/mod.rs`
- ✅ All existing tests pass
- ✅ `src/lib.rs` reduced to orchestration only

**Files Created/Modified**:
- `src/parser/mod.rs`, `src/parser/hayagriva_parser.rs`
- `src/backend/mod.rs`, `src/backend/legacy.rs`
- `src/models/mod.rs`
- `src/citation/mod.rs`
- `src/renderer/mod.rs`
- `src/io/mod.rs`

---

### Phase 2: Hayagriva Parser Integration ✅ COMPLETED

**Goal**: Replace nom-bibtex with hayagriva for parsing.

**Achievements**:

- ✅ Added `hayagriva = "0.9"` dependency with `archive` feature
- ✅ Implemented hayagriva parser in `src/parser/hayagriva_parser.rs`
- ✅ Support for both BibTeX and YAML bibliography formats
- ✅ Better author parsing using hayagriva's `Person` objects
- ✅ Removed preprocessing hacks (quote removal, R@10 pattern replacement)
- ✅ All existing tests pass with hayagriva parser

**Files Modified**:
- `Cargo.toml` - Added hayagriva dependency
- `src/parser/hayagriva_parser.rs` - New implementation
- `src/lib.rs` - Updated to use new parser

---

### Phase 3: BibItem Expansion ✅ COMPLETED

**Goal**: Expand BibItem struct with all hayagriva fields.

**Achievements**:

- ✅ Added `hayagriva_entry: Option<Arc<Entry>>` for CSL rendering
- ✅ Added new fields: `entry_type`, `doi`, `pages`, `volume`, `issue`, `publisher`, `address`, `editor`, `edition`, `series`, `note`
- ✅ Updated hayagriva parser to populate all new fields
- ✅ All fields are `Option` or `Vec` (backwards compatible)
- ✅ Serialization/deserialization works correctly

**Files Modified**:
- `src/models/mod.rs` - Expanded BibItem struct
- `src/parser/hayagriva_parser.rs` - Map all fields from Entry

---

### Phase 4: Backend Abstraction ✅ COMPLETED

**Goal**: Implement `BibliographyBackend` trait with LegacyBackend and CslBackend.

**Achievements**:

- ✅ Defined `BibliographyBackend` trait in `src/backend/mod.rs`
- ✅ Implemented `LegacyBackend` in `src/backend/legacy.rs`:
  - Uses Handlebars registry for rendering
  - Preserves exact current behavior
- ✅ Created `CslBackend` stub in `src/backend/csl.rs`
- ✅ Added `BackendMode` enum (Legacy, Csl)
- ✅ Updated `Config` to detect backend mode
- ✅ Updated `Bibliography::run()` to use backend abstraction

**Files Created/Modified**:
- `src/backend/mod.rs` - Trait definition + BackendMode enum
- `src/backend/legacy.rs` - LegacyBackend implementation
- `src/backend/csl.rs` - CslBackend stub
- `src/config.rs` - Backend mode detection
- `src/lib.rs` - Use backend abstraction

---

### Phase 5: Full CSL Integration ✅ COMPLETED

**Goal**: Full CSL backend implementation with hayagriva.

**Achievements**:

- ✅ Enabled hayagriva `archive` feature for 80+ bundled CSL styles
- ✅ Implemented `CslBackend` with full `BibliographyDriver` integration
- ✅ Added style name aliases (ieee, apa, chicago-author-date, nature, etc.)
- ✅ Implemented `format_citation()`:
  - Numeric styles (IEEE, Nature): Use `item.index` for proper sequential numbering
  - Author-date styles (Chicago, APA): Use hayagriva CSL formatting
  - Superscript styles (Nature): Render as `<sup>` tags
- ✅ Implemented `format_reference()` with proper CSL formatting
- ✅ Added citation→bibliography linking with anchor IDs
- ✅ Fixed ANSI escape code stripping from hayagriva output
- ✅ Created three CSL test books:
  - `test_book_csl_ieee/` - IEEE numbered citations [1], [2], [3]
  - `test_book_csl_chicago/` - Chicago author-date (Author Year)
  - `test_book_csl_nature/` - Nature superscript citations
- ✅ All 34 tests passing

**Supported CSL Styles**:
- Numeric: ieee, nature, vancouver, vancouver-superscript, acm, acs, ama, cell, springer-basic
- Author-date: chicago-author-date, apa, mla, harvard, elsevier-harvard
- And 70+ more via hayagriva's archive

**Files Created/Modified**:
- `src/backend/csl.rs` - Full CslBackend implementation
- `Cargo.toml` - Enabled `archive` feature
- `test_book_csl_ieee/` - IEEE test book
- `test_book_csl_chicago/` - Chicago test book
- `test_book_csl_nature/` - Nature test book

---

### Phase 6: Testing, Documentation, Polish 🔄 IN PROGRESS

**Goal**: Comprehensive testing, documentation, and release preparation.

**Documentation Updates** ✅ COMPLETED:

- ✅ Updated `README.md`:
  - Added dual backend feature description
  - Added "Rendering Backends" section with context
  - Added quick start examples for both backends
  - Added links to detailed manual pages
- ✅ Updated `manual/src/intro.md`:
  - Added backend comparison table
  - Added quick start section
  - Links to detailed backend pages
- ✅ Created `manual/src/legacy.md`:
  - Complete template variables reference
  - Example templates for bibliography and citations
  - Custom CSS/JS examples
- ✅ Created `manual/src/csl.md`:
  - Available CSL styles (numeric, author-date, note)
  - Examples for IEEE, Chicago, Nature, APA
  - YAML bibliography support documentation
- ✅ Updated `manual/src/config.md`:
  - Simplified structure with backend selection
  - Configuration reference table
  - Complete examples for different use cases
- ✅ Updated `manual/src/SUMMARY.md` with new pages

**Remaining Tasks**:

1. Create comprehensive test suite:
   - Regression tests (old output vs new output)
   - Backend-specific tests (Legacy vs CSL)
   - YAML bibliography tests
   - Zotero integration tests
   - Per-chapter bibliography tests with all backends
   - Edge cases (missing fields, malformed entries)

2. Write migration guide:
   - How to upgrade from v0.5.x
   - When to use Legacy vs CSL backend

3. Performance benchmarking:
   - Compare to nom-bibtex baseline
   - Test with large bibliographies

4. Security audit:
   - Dependency check
   - Input validation

**Success Criteria**:

- [ ] All tests pass (target: 95%+ code coverage)
- [x] Documentation complete and clear
- [ ] Migration guide covers common scenarios
- [ ] Performance equal or better than v0.5.x
- [ ] No known security issues
- [ ] Ready for release

---

## Testing Strategy

### Test Books

| Directory | Backend | Purpose |
|-----------|---------|---------|
| `test_book/` | Legacy | Original test book, backwards compatibility |
| `test_book_csl_ieee/` | CSL | IEEE numbered citations |
| `test_book_csl_chicago/` | CSL | Chicago author-date citations |
| `test_book_csl_nature/` | CSL | Nature superscript citations |

### Running Tests

```bash
# Run all tests
cargo test

# Run with logging
MDBOOK_LOG=mdbook_bib=debug cargo test

# Build test books
cd test_book && mdbook build
cd test_book_csl_ieee && mdbook build
cd test_book_csl_chicago && mdbook build
cd test_book_csl_nature && mdbook build
```

## Backwards Compatibility

### Guarantees

1. **Existing books build identically** - No config changes required
2. **Custom templates continue working** - All existing template variables available
3. **Configuration is additive** - No options removed
4. **Citation syntaxes unchanged** - `{{#cite key}}` and `@@key` work identically

### Migration Path

**For existing users:**
1. Upgrade to v1.0.0
2. No config changes needed (uses Legacy mode automatically)
3. Optionally try CSL backend: Add `backend = "csl"` and `csl-style = "ieee"`

**For new users:**
- Legacy mode for full customization
- CSL mode for standard academic formats

## Conclusion

This integration provides two complementary backend modes:

### Legacy Backend (Handlebars)
- Full template customization
- Interactive elements (copy buttons, collapsible details)
- Best for power users with specific formatting needs

### CSL Backend (hayagriva)
- Standard academic citation styles (IEEE, Chicago, Nature, APA, etc.)
- Proper numeric citation numbering
- Superscript support for Nature-style citations
- Best for users who want standard formats without template work

Both backends share the same hayagriva-powered parser, ensuring consistent bibliography parsing across modes.
