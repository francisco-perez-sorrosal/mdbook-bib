//! Tests for bibliography rendering backends.
//!
//! This module covers:
//! - Custom backend (Handlebars-based)
//! - CSL backend (various styles)
//! - Backend comparison tests
//! - Regression tests for output formats

use super::common::{
    create_citation_backend, create_references_backend, dummy_bibliography, yaml_bibliography,
};
use crate::backend::{BibliographyBackend, CitationContext, CslBackend};
use crate::config::SortOrder;
use crate::parser::{self, BibFormat};
use mdbook_preprocessor::book::Chapter;
use rstest::rstest;
use std::collections::HashSet;

// =============================================================================
// Custom Backend Regression Tests
// =============================================================================

#[test]
fn regression_custom_citation_format() {
    // Verify Custom backend produces same citation format as before hayagriva integration
    let mut bibliography = dummy_bibliography();

    let chapter = Chapter::new(
        "Test",
        "Reference to {{#cite fps}} here.".to_string(),
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

    // Custom format: <a class="bib-cite" href="bibliography.html#key">key</a>
    assert!(
        result.contains(r#"href="bibliography.html#fps""#),
        "Custom citation should link to bibliography: {result}"
    );
}

#[test]
fn regression_custom_bibliography_html_structure() {
    // Verify Custom backend produces expected bibliography HTML structure
    let bibliography = dummy_bibliography();
    let backend = create_references_backend();

    let html = crate::renderer::generate_bibliography_html(
        &bibliography,
        &HashSet::new(),
        false, // render all
        &backend,
        SortOrder::None,
    );

    // Verify expected structure elements
    assert!(html.contains("bib-entry"), "Should have bib-entry class");
    assert!(
        html.contains("This is a bib entry!"),
        "Should contain fps title"
    );
    assert!(
        html.contains("The Rust Programming Language"),
        "Should contain rust_book title"
    );
}

#[test]
fn bibliography_render_all_vs_cited() {
    let bibliography = dummy_bibliography();

    let mut cited = HashSet::new();
    cited.insert("fps".to_string());

    let backend = create_references_backend();

    let html = crate::renderer::generate_bibliography_html(
        &bibliography,
        &cited,
        false,
        &backend,
        SortOrder::None,
    );

    assert!(html.contains("This is a bib entry!"));
    assert!(html.contains("The Rust Programming Language"));

    let html = crate::renderer::generate_bibliography_html(
        &bibliography,
        &cited,
        true,
        &backend,
        SortOrder::None,
    );

    assert!(html.contains("This is a bib entry!"));
    assert!(!html.contains("The Rust Programming Language"));
}

#[test]
fn bibliography_renders_url_when_present() {
    let bibliography = dummy_bibliography();
    let backend = create_references_backend();

    let html = crate::renderer::generate_bibliography_html(
        &bibliography,
        &HashSet::new(),
        false,
        &backend,
        SortOrder::None,
    );
    assert!(html.contains("href=\"https://doc.rust-lang.org/book/\""));
}

// =============================================================================
// Custom vs CSL Backend Comparison Tests
// =============================================================================

#[test]
fn backend_custom_vs_csl_citation_format_differs() {
    let bib_src = r#"
@article{smith2024,
    author = {Smith, John},
    title = {A Test Article},
    journal = {Test Journal},
    year = {2024},
}
"#;

    let bibliography = parser::parse_bibliography(bib_src.to_string(), BibFormat::BibTeX).unwrap();
    let item = bibliography.get("smith2024").unwrap();

    let context = CitationContext {
        bib_page_path: "bibliography.html".to_string(),
        chapter_path: "chapter.md".to_string(),
    };

    // Custom backend
    let custom_backend = create_citation_backend();
    let custom_citation = custom_backend.format_citation(item, &context).unwrap();

    // CSL backend (IEEE - numeric style)
    let csl_backend = CslBackend::new("ieee".to_string()).unwrap();
    let csl_citation = csl_backend.format_citation(item, &context).unwrap();

    // Both should produce valid output but different formats
    assert!(
        !custom_citation.is_empty(),
        "Custom citation should not be empty"
    );
    assert!(!csl_citation.is_empty(), "CSL citation should not be empty");

    // Custom uses [key] format, CSL uses [number] format
    assert!(
        custom_citation.contains("smith2024"),
        "Custom should use citation key"
    );
    // CSL IEEE uses numbered citations
    assert!(
        csl_citation.contains("[[") || csl_citation.contains("<a href"),
        "CSL should have link: {csl_citation}"
    );
}

// =============================================================================
// CSL Backend Style Tests (Parametrized)
// =============================================================================

const CSL_TEST_BIB: &str = r#"
@article{test_entry,
    author = {Smith, John},
    title = {Test Article},
    journal = {Test Journal},
    year = {2024},
}
"#;

#[rstest]
#[case::ieee("ieee")]
#[case::apa("apa")]
#[case::chicago_author_date("chicago-author-date")]
#[case::nature("nature")]
fn backend_csl_citation_links_to_bibliography(#[case] style: &str) {
    let bibliography =
        parser::parse_bibliography(CSL_TEST_BIB.to_string(), BibFormat::BibTeX).unwrap();
    let item = bibliography.get("test_entry").unwrap();

    let context = CitationContext {
        bib_page_path: "bibliography.html".to_string(),
        chapter_path: "chapter.md".to_string(),
    };

    let backend = CslBackend::new(style.to_string()).unwrap();
    let citation = backend.format_citation(item, &context).unwrap();

    assert!(
        citation.contains("bibliography.html"),
        "{style} citation should link to bibliography: {citation}"
    );
}

#[rstest]
#[case::ieee("ieee")]
#[case::apa("apa")]
#[case::chicago_author_date("chicago-author-date")]
#[case::nature("nature")]
fn backend_csl_reference_has_entry_class(#[case] style: &str) {
    let bibliography =
        parser::parse_bibliography(CSL_TEST_BIB.to_string(), BibFormat::BibTeX).unwrap();
    let item = bibliography.get("test_entry").unwrap();

    let backend = CslBackend::new(style.to_string()).unwrap();
    let reference = backend.format_reference(item).unwrap();

    assert!(
        reference.contains("class='csl-entry'"),
        "{style} should have csl-entry class: {reference}"
    );
    assert!(
        reference.contains("id='test_entry'"),
        "{style} should have citation key id: {reference}"
    );
}

#[test]
fn backend_csl_nature_uses_superscript() {
    let bibliography =
        parser::parse_bibliography(CSL_TEST_BIB.to_string(), BibFormat::BibTeX).unwrap();
    let item = bibliography.get("test_entry").unwrap();

    let context = CitationContext {
        bib_page_path: "bibliography.html".to_string(),
        chapter_path: "chapter.md".to_string(),
    };

    let nature_backend = CslBackend::new("nature".to_string()).unwrap();
    let citation = nature_backend.format_citation(item, &context).unwrap();

    assert!(
        citation.contains("<sup>"),
        "Nature should use superscript: {citation}"
    );
}

// =============================================================================
// YAML Bibliography with Backends Tests
// =============================================================================

#[test]
fn yaml_bibliography_with_custom_backend() {
    let bibliography = yaml_bibliography();
    let backend = create_references_backend();

    let html = crate::renderer::generate_bibliography_html(
        &bibliography,
        &HashSet::new(),
        false,
        &backend,
        SortOrder::None,
    );

    assert!(
        html.contains("A YAML Bibliography Entry"),
        "Should contain YAML entry title"
    );
    assert!(
        html.contains("The Complete Guide to Bibliography Systems"),
        "Should contain book title"
    );
}

#[test]
fn yaml_bibliography_with_csl_backend() {
    let bibliography = yaml_bibliography();
    let item = bibliography.get("smith2024").unwrap();

    let csl_backend = CslBackend::new("apa".to_string()).unwrap();
    let reference = csl_backend.format_reference(item);

    assert!(
        reference.is_ok(),
        "CSL should render YAML entry: {:?}",
        reference.err()
    );
    let ref_html = reference.unwrap();
    assert!(
        ref_html.contains("csl-entry"),
        "Should have CSL entry class"
    );
}
