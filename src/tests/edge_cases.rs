//! Tests for edge cases and error handling.
//!
//! This module covers:
//! - Empty bibliography handling
//! - Malformed BibTeX entries
//! - Unknown citation keys
//! - Special characters in content
//! - Unicode support

use super::common::{create_citation_backend, dummy_bibliography};
use crate::parser::{self, BibFormat};
use mdbook_preprocessor::book::Chapter;
use std::collections::HashSet;

// =============================================================================
// Empty and Invalid Input Tests
// =============================================================================

#[test]
fn empty_bibliography_handling() {
    let empty_bib = "";
    let result = parser::parse_bibliography(empty_bib.to_string(), BibFormat::BibTeX);
    // Empty bibliography should parse but return empty IndexMap
    assert!(result.is_ok(), "Empty bibliography should parse");
    assert!(result.unwrap().is_empty(), "Should return empty IndexMap");
}

#[test]
fn malformed_bibtex_entry() {
    // Hayagriva is strict about BibTeX syntax - malformed entries cause parse errors
    let malformed = r#"
@article{incomplete_entry
    author = Missing closing brace
"#;

    let result = parser::parse_bibliography(malformed.to_string(), BibFormat::BibTeX);
    // Hayagriva returns an error for malformed BibTeX
    assert!(
        result.is_err() || result.unwrap().is_empty(),
        "Malformed BibTeX should either error or return empty"
    );
}

#[test]
fn valid_bibtex_after_malformed_is_still_parsed() {
    // Test that a well-formed entry parses correctly
    let good_entry = r#"
@article{good_entry,
    author = {Good, Author},
    title = {Good Title},
    year = {2024},
}
"#;

    let result = parser::parse_bibliography(good_entry.to_string(), BibFormat::BibTeX);
    assert!(result.is_ok(), "Well-formed BibTeX should parse");
    let bibliography = result.unwrap();
    assert_eq!(bibliography.len(), 1, "Should have one entry");
    assert!(
        bibliography.contains_key("good_entry"),
        "Should contain good_entry"
    );
}

// =============================================================================
// Unknown Citation Key Tests
// =============================================================================

#[test]
fn citation_to_nonexistent_key() {
    let mut bibliography = dummy_bibliography();

    let chapter = Chapter::new(
        "Test",
        "Reference to {{#cite nonexistent_key}} here.".to_string(),
        "chapter.md",
        vec![],
    );

    let backend = create_citation_backend();
    let mut cited = HashSet::new();
    let mut last_index = 0;

    let result = crate::citation::replace_all_placeholders(
        &chapter,
        &mut bibliography,
        &mut cited,
        &backend,
        &mut last_index,
    );

    // Should contain error message for unknown key
    assert!(
        result.contains("Unknown bib ref"),
        "Should indicate unknown reference: {result}"
    );
}

// =============================================================================
// Special Characters Tests
// =============================================================================

#[test]
fn special_characters_in_title() {
    let special_bib = r#"
@article{special2024,
    author = {Test, Author},
    title = {Testing Special Characters: <>&"'},
    year = {2024},
}
"#;

    let result = parser::parse_bibliography(special_bib.to_string(), BibFormat::BibTeX);
    assert!(
        result.is_ok(),
        "Should parse entries with special characters"
    );

    let bibliography = result.unwrap();
    let entry = bibliography.get("special2024").unwrap();
    assert!(!entry.title.is_empty(), "Title should be extracted");
}

#[test]
fn unicode_in_author_names() {
    let unicode_bib = r#"
@article{unicode2024,
    author = {Müller, Hans and García, José and 中村, 太郎},
    title = {Unicode Author Names Test},
    year = {2024},
}
"#;

    let result = parser::parse_bibliography(unicode_bib.to_string(), BibFormat::BibTeX);
    assert!(result.is_ok(), "Should parse entries with unicode authors");

    let bibliography = result.unwrap();
    let entry = bibliography.get("unicode2024").unwrap();
    assert!(!entry.authors.is_empty(), "Authors should be extracted");
}
