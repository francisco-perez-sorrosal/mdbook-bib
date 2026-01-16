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
// CSL Backend Style Tests
// =============================================================================

#[test]
fn backend_csl_numeric_vs_author_date() {
    let bib_src = r#"
@article{smith2024,
    author = {Smith, John},
    title = {Test Article},
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

    // IEEE (numeric)
    let ieee_backend = CslBackend::new("ieee".to_string()).unwrap();
    let ieee_citation = ieee_backend.format_citation(item, &context).unwrap();

    // Chicago author-date
    let chicago_backend = CslBackend::new("chicago-author-date".to_string()).unwrap();
    let chicago_citation = chicago_backend.format_citation(item, &context).unwrap();

    // IEEE uses numbers, Chicago uses author-date
    println!("IEEE citation: {ieee_citation}");
    println!("Chicago citation: {chicago_citation}");

    // Both should contain links
    assert!(
        ieee_citation.contains("bibliography.html"),
        "IEEE should link to bibliography"
    );
    assert!(
        chicago_citation.contains("bibliography.html"),
        "Chicago should link to bibliography"
    );
}

#[test]
fn backend_csl_superscript_style() {
    let bib_src = r#"
@article{watson1953,
    author = {Watson, James},
    title = {DNA Structure},
    journal = {Nature},
    year = {1953},
}
"#;

    let bibliography = parser::parse_bibliography(bib_src.to_string(), BibFormat::BibTeX).unwrap();
    let item = bibliography.get("watson1953").unwrap();

    let context = CitationContext {
        bib_page_path: "bibliography.html".to_string(),
        chapter_path: "chapter.md".to_string(),
    };

    // Nature uses superscript
    let nature_backend = CslBackend::new("nature".to_string()).unwrap();
    let nature_citation = nature_backend.format_citation(item, &context).unwrap();

    println!("Nature citation: {nature_citation}");

    // Nature should use <sup> tags
    assert!(
        nature_citation.contains("<sup>"),
        "Nature should use superscript: {nature_citation}"
    );
}

#[test]
fn backend_csl_reference_format() {
    let bib_src = r#"
@article{smith2024,
    author = {Smith, John and Doe, Jane},
    title = {Research Methods in Computer Science},
    journal = {Journal of CS},
    year = {2024},
    volume = {10},
    pages = {1-20},
}
"#;

    let bibliography = parser::parse_bibliography(bib_src.to_string(), BibFormat::BibTeX).unwrap();
    let item = bibliography.get("smith2024").unwrap();

    // IEEE reference
    let ieee_backend = CslBackend::new("ieee".to_string()).unwrap();
    let ieee_ref = ieee_backend.format_reference(item).unwrap();

    // APA reference
    let apa_backend = CslBackend::new("apa".to_string()).unwrap();
    let apa_ref = apa_backend.format_reference(item).unwrap();

    println!("IEEE reference: {ieee_ref}");
    println!("APA reference: {apa_ref}");

    // Both should have the csl-entry class and citation key id
    assert!(
        ieee_ref.contains("class='csl-entry'"),
        "IEEE should have csl-entry class"
    );
    assert!(
        ieee_ref.contains("id='smith2024'"),
        "IEEE should have citation key id"
    );
    assert!(
        apa_ref.contains("class='csl-entry'"),
        "APA should have csl-entry class"
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
