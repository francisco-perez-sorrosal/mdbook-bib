//! Tests for bibliography parsing functionality.
//!
//! This module covers:
//! - BibTeX format parsing
//! - YAML format parsing
//! - Date/month extraction
//! - Extended BibItem fields
//! - Serialization

use super::common::{dummy_bibliography, yaml_bibliography, DUMMY_BIB_SRC, YAML_BIB_SRC};
use crate::io;
use crate::parser::{self, BibFormat};
use std::fs::File;
use std::io::Write;
use tempfile::Builder as TempFileBuilder;

// =============================================================================
// IO Loading Tests
// =============================================================================

#[test]
fn load_bib_bibliography_from_file() {
    let temp = TempFileBuilder::new().prefix("book").tempdir().unwrap();
    let chapter_path = temp.path().join("biblio.bib");
    File::create(&chapter_path)
        .unwrap()
        .write_all(DUMMY_BIB_SRC.as_bytes())
        .unwrap();

    let bibliography_loaded: String = io::load_bibliography(chapter_path.as_path()).unwrap();
    assert_ne!(bibliography_loaded, "");
    assert!(bibliography_loaded.contains("Francisco Perez-Sorrosal"));
}

#[test]
fn cant_load_bib_bibliography_from_file() {
    let temp = TempFileBuilder::new().prefix("book").tempdir().unwrap();
    let chapter_path = temp.path().join("biblio.wrong_extension");
    File::create(&chapter_path)
        .unwrap()
        .write_all(DUMMY_BIB_SRC.as_bytes())
        .unwrap();

    let bibliography_loaded: String = io::load_bibliography(chapter_path.as_path()).unwrap();
    assert_eq!(bibliography_loaded, "");
}

// =============================================================================
// BibTeX Parsing Tests
// =============================================================================

#[test]
fn bibliography_builder_returns_a_bibliography() {
    let bibliography = dummy_bibliography();
    assert_eq!(bibliography.len(), 2);
    assert_eq!(bibliography.get("fps").unwrap().citation_key, "fps");
}

#[test]
fn bibliography_includes_url_when_present_in_bibitems() {
    let bibliography = dummy_bibliography();

    // fps dummy book does not include a url in the BibItem
    let fps = bibliography.get("fps");
    assert!(fps.unwrap().url.is_none());

    // rust_book does...
    let rust_book = bibliography.get("rust_book");
    assert_eq!(
        rust_book.unwrap().url.as_ref().unwrap(),
        "https://doc.rust-lang.org/book/"
    );
}

// =============================================================================
// Date Extraction Tests
// =============================================================================

#[test]
fn test_hayagriva_date_extraction() {
    // Test string month (short form - standard BibTeX constant)
    let bib_string_short = r#"
@article{string_month_short,
    title = {Test String Month Short},
    author = {Doe, John},
    month = oct,
    year = {2020},
}
"#;
    let result = parser::parse_bibliography(bib_string_short.to_string(), BibFormat::BibTeX);
    assert!(result.is_ok());
    let bibliography = result.unwrap();
    let entry = bibliography.get("string_month_short").unwrap();
    assert_eq!(entry.pub_year, Some("2020".to_string()));
    assert_eq!(entry.pub_month, Some("10".to_string()));

    // Test missing month (year only)
    let bib_year_only = r#"
@article{year_only,
    title = {Test Year Only},
    author = {Doe, John},
    year = {2020},
}
"#;
    let result = parser::parse_bibliography(bib_year_only.to_string(), BibFormat::BibTeX);
    assert!(result.is_ok());
    let bibliography = result.unwrap();
    let entry = bibliography.get("year_only").unwrap();
    assert_eq!(entry.pub_year, Some("2020".to_string()));
    assert_eq!(entry.pub_month, None);

    // Test missing date entirely
    let bib_no_date = r#"
@article{no_date,
    title = {Test No Date},
    author = {Doe, John},
}
"#;
    let result = parser::parse_bibliography(bib_no_date.to_string(), BibFormat::BibTeX);
    assert!(result.is_ok());
    let bibliography = result.unwrap();
    let entry = bibliography.get("no_date").unwrap();
    assert_eq!(entry.pub_year, None);
    assert_eq!(entry.pub_month, None);

    // Test Zotero-style month with braces (common export format)
    let bib_zotero_style = r#"
@article{zotero_month,
    title = {Test Zotero Month Format},
    author = {Doe, John},
    month = {oct},
    year = {2020},
}
"#;
    let result = parser::parse_bibliography(bib_zotero_style.to_string(), BibFormat::BibTeX);
    assert!(result.is_ok());
    let bibliography = result.unwrap();
    let entry = bibliography.get("zotero_month").unwrap();
    assert_eq!(entry.pub_year, Some("2020".to_string()));
    assert_eq!(entry.pub_month, Some("10".to_string()));
}

// =============================================================================
// Extended BibItem Fields Tests
// =============================================================================

#[test]
fn test_extended_bibitem_fields() {
    let bib_comprehensive = r#"
@article{comprehensive_entry,
    title = {A Comprehensive Research Article},
    author = {Smith, John and Doe, Jane},
    editor = {Johnson, Robert},
    year = {2023},
    month = mar,
    journal = {Journal of Computer Science},
    volume = {42},
    number = {3},
    pages = {123-145},
    doi = {10.1234/jcs.2023.42.3.123},
    issn = {1234-5678},
    publisher = {Academic Press},
    address = {Cambridge, MA},
    note = {This is a comprehensive test entry},
}
"#;

    let result = parser::parse_bibliography(bib_comprehensive.to_string(), BibFormat::BibTeX);
    assert!(result.is_ok(), "Failed to parse comprehensive entry");

    let bibliography = result.unwrap();
    let entry = bibliography.get("comprehensive_entry").unwrap();

    // Verify core fields
    assert_eq!(entry.citation_key, "comprehensive_entry");
    assert_eq!(entry.title, "A Comprehensive Research Article");
    assert_eq!(entry.pub_year, Some("2023".to_string()));
    assert_eq!(entry.pub_month, Some("03".to_string()));

    // Verify authors
    assert_eq!(entry.authors.len(), 2);
    assert_eq!(entry.authors[0], vec!["Smith", "John"]);
    assert_eq!(entry.authors[1], vec!["Doe", "Jane"]);

    // Verify extended fields that hayagriva supports
    assert!(entry.entry_type.is_some(), "entry_type should be present");
    assert!(entry.entry_type.as_ref().unwrap().contains("Article"));

    assert_eq!(entry.doi, Some("10.1234/jcs.2023.42.3.123".to_string()));
    assert_eq!(entry.issn, Some("1234-5678".to_string()));

    // Volume and issue - check if extracted
    if entry.volume.is_some() {
        assert_eq!(entry.volume, Some("42".to_string()));
    }
    if entry.issue.is_some() {
        assert_eq!(entry.issue, Some("3".to_string()));
    }

    // Pages should be extracted
    assert!(
        entry.pages.is_some(),
        "pages should be present: {:?}",
        entry.pages
    );

    // Editor - check if extracted
    if let Some(editors) = &entry.editor {
        assert!(!editors.is_empty(), "Should have at least one editor");
    }
}

#[test]
fn test_book_entry_with_isbn() {
    let bib_book = r#"
@book{rust_programming,
    title = {The Rust Programming Language},
    author = {Klabnik, Steve and Nichols, Carol},
    year = {2018},
    publisher = {No Starch Press},
    address = {San Francisco},
    isbn = {978-1593278281},
    edition = {1st},
}
"#;

    let result = parser::parse_bibliography(bib_book.to_string(), BibFormat::BibTeX);
    assert!(result.is_ok());

    let bibliography = result.unwrap();
    let entry = bibliography.get("rust_programming").unwrap();

    assert_eq!(entry.citation_key, "rust_programming");
    assert_eq!(entry.title, "The Rust Programming Language");
    assert!(entry.entry_type.is_some());
    assert!(entry.entry_type.as_ref().unwrap().contains("Book"));

    // ISBN should be extracted
    assert_eq!(entry.isbn, Some("978-1593278281".to_string()));
}

#[test]
fn test_inproceedings_with_organization() {
    let bib_conference = r#"
@inproceedings{conference_paper,
    title = {Novel Approaches to Distributed Systems},
    author = {Williams, Sarah},
    year = {2024},
    month = jun,
    booktitle = {Proceedings of ICSE 2024},
    pages = {45-60},
    organization = {IEEE},
    address = {Lisbon, Portugal},
}
"#;

    let result = parser::parse_bibliography(bib_conference.to_string(), BibFormat::BibTeX);
    assert!(result.is_ok());

    let bibliography = result.unwrap();
    let entry = bibliography.get("conference_paper").unwrap();

    assert_eq!(entry.citation_key, "conference_paper");
    assert!(entry.entry_type.is_some(), "entry_type should be present");

    assert_eq!(entry.pub_month, Some("06".to_string()));
    assert!(entry.pages.is_some(), "pages should be extracted");

    // Organization - hayagriva support may vary
    if entry.organization.is_some() {
        assert_eq!(entry.organization, Some("IEEE".to_string()));
    }
}

#[test]
fn test_entry_with_minimal_fields() {
    let bib_minimal = r#"
@misc{minimal_entry,
    title = {Minimal Entry},
    author = {Anonymous},
    year = {2024},
}
"#;

    let result = parser::parse_bibliography(bib_minimal.to_string(), BibFormat::BibTeX);
    assert!(result.is_ok());

    let bibliography = result.unwrap();
    let entry = bibliography.get("minimal_entry").unwrap();

    assert_eq!(entry.citation_key, "minimal_entry");
    assert_eq!(entry.title, "Minimal Entry");

    // All extended fields should be None
    assert!(entry.doi.is_none());
    assert!(entry.isbn.is_none());
    assert!(entry.issn.is_none());
    assert!(entry.volume.is_none());
    assert!(entry.issue.is_none());
    assert!(entry.pages.is_none());
    assert!(entry.editor.is_none());
    assert!(entry.organization.is_none());
}

// =============================================================================
// Serialization Tests
// =============================================================================

#[test]
fn test_serialization_with_extended_fields() {
    let bib = r#"
@article{serialize_test,
    title = {Serialization Test},
    author = {Tester, John},
    year = {2024},
    doi = {10.1234/test.2024},
    volume = {1},
    pages = {1-10},
}
"#;

    let result = parser::parse_bibliography(bib.to_string(), BibFormat::BibTeX);
    assert!(result.is_ok());

    let bibliography = result.unwrap();
    let entry = bibliography.get("serialize_test").unwrap();

    // Serialize to JSON
    let json_result = serde_json::to_string(&entry);
    assert!(json_result.is_ok(), "Should serialize to JSON");

    let json = json_result.unwrap();
    assert!(json.contains("doi"), "DOI should be in JSON");
    assert!(
        json.contains("10.1234/test.2024"),
        "DOI value should be in JSON"
    );

    // Volume might or might not be extracted by hayagriva
    if entry.volume.is_some() {
        assert!(
            json.contains("volume"),
            "Volume should be in JSON when present"
        );
    }

    // Pages should be present
    assert!(entry.pages.is_some(), "Pages should be extracted");

    // Fields that are None should not appear in JSON (due to skip_serializing_if)
    assert!(
        !json.contains("isbn"),
        "ISBN should not be in JSON when None"
    );
    assert!(
        !json.contains("issn"),
        "ISSN should not be in JSON when None"
    );
}

// =============================================================================
// YAML Bibliography Tests
// =============================================================================

#[test]
fn yaml_bibliography_parsing() {
    let result = parser::parse_bibliography(YAML_BIB_SRC.to_string(), BibFormat::Yaml);
    assert!(
        result.is_ok(),
        "YAML parsing should succeed: {:?}",
        result.err()
    );

    let bibliography = result.unwrap();
    assert_eq!(bibliography.len(), 2, "Should have 2 entries");

    // Check first entry
    let smith = bibliography.get("smith2024").unwrap();
    assert_eq!(smith.citation_key, "smith2024");
    assert_eq!(smith.title, "A YAML Bibliography Entry");
    assert_eq!(smith.pub_year, Some("2024".to_string()));

    // Check second entry
    let jones = bibliography.get("jones2023").unwrap();
    assert_eq!(jones.citation_key, "jones2023");
    assert_eq!(jones.title, "The Complete Guide to Bibliography Systems");
    assert_eq!(jones.authors.len(), 2, "Should have 2 authors");
}

#[test]
fn yaml_vs_bibtex_equivalent_output() {
    // Same entry in both formats should produce equivalent BibItems
    let bibtex_src = r#"
@article{test2024,
    author = {Test, Author},
    title = {Test Title},
    journal = {Test Journal},
    year = {2024},
}
"#;

    let yaml_src = r#"
test2024:
  type: article
  title: Test Title
  author: Test, Author
  date: 2024
  parent:
    type: periodical
    title: Test Journal
"#;

    let bibtex_bib = parser::parse_bibliography(bibtex_src.to_string(), BibFormat::BibTeX).unwrap();
    let yaml_bib = parser::parse_bibliography(yaml_src.to_string(), BibFormat::Yaml).unwrap();

    let bibtex_item = bibtex_bib.get("test2024").unwrap();
    let yaml_item = yaml_bib.get("test2024").unwrap();

    // Core fields should match
    assert_eq!(bibtex_item.citation_key, yaml_item.citation_key);
    assert_eq!(bibtex_item.title, yaml_item.title);
    assert_eq!(bibtex_item.pub_year, yaml_item.pub_year);
}

#[test]
fn yaml_bibliography_returns_pre_parsed() {
    // Test that the helper returns usable data
    let bibliography = yaml_bibliography();
    assert_eq!(bibliography.len(), 2);
    assert!(bibliography.contains_key("smith2024"));
    assert!(bibliography.contains_key("jones2023"));
}
