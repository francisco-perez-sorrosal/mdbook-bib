//! Integration tests for full book processing.
//!
//! This module covers:
//! - Complete test book builds
//! - CSL style book builds (IEEE, Chicago, Nature)
//! - Output verification

use super::common::find_str_in_file;
use crate::Bibliography;
use mdbook_driver::MDBook;
use std::path::PathBuf;

// =============================================================================
// Test Book Integration Tests
// =============================================================================

#[test]
fn process_test_book() {
    let mut manual_src_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manual_src_path.push("test_book/");
    let mut md = MDBook::load(manual_src_path).unwrap();
    let mdbook_bib_prepro = Bibliography;
    md.with_preprocessor(mdbook_bib_prepro);
    match md.build() {
        Ok(_) => (),
        Err(err) => panic!("there was an error building the book: {err:?}"),
    }

    // Check both, root level and nested html files get placeholders substituted with
    // bib references with relative paths
    let mut book_dest_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    book_dest_path.push("test_book/public");

    let bib_reference = "bibliography.html#mdBook";

    let mut non_nested_html = book_dest_path.clone();
    non_nested_html.push("intro.html");
    match find_str_in_file(bib_reference, non_nested_html) {
        Ok(_) => (),
        Err(_) => panic!(),
    }

    let mut nested_html = book_dest_path.clone();
    nested_html.push("chapter_1/intro.html");
    match find_str_in_file(bib_reference, nested_html) {
        Ok(_) => (),
        Err(_) => panic!(),
    }
}

// =============================================================================
// CSL Style Integration Tests
// =============================================================================

#[test]
fn process_test_book_csl_ieee() {
    // Integration test for CSL IEEE test book
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("test_book_csl_ieee/");

    let mut md = MDBook::load(path).unwrap();
    let mdbook_bib_prepro = Bibliography;
    md.with_preprocessor(mdbook_bib_prepro);

    match md.build() {
        Ok(_) => (),
        Err(err) => panic!("Error building CSL IEEE test book: {err:?}"),
    }

    // Verify output contains CSL-formatted citations
    let mut output_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    output_path.push("test_book_csl_ieee/book");

    // Check that the bibliography page exists
    let mut bib_path = output_path.clone();
    bib_path.push("bibliography.html");
    assert!(bib_path.exists(), "Bibliography page should exist");

    // Verify CSL entry class is present
    match find_str_in_file("csl-entry", bib_path) {
        Ok(_) => (),
        Err(_) => panic!("CSL entry class not found in bibliography"),
    }
}

#[test]
fn process_test_book_csl_chicago() {
    // Integration test for CSL Chicago test book
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("test_book_csl_chicago/");

    let mut md = MDBook::load(path).unwrap();
    let mdbook_bib_prepro = Bibliography;
    md.with_preprocessor(mdbook_bib_prepro);

    match md.build() {
        Ok(_) => (),
        Err(err) => panic!("Error building CSL Chicago test book: {err:?}"),
    }
}

#[test]
fn process_test_book_csl_nature() {
    // Integration test for CSL Nature test book
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("test_book_csl_nature/");

    let mut md = MDBook::load(path).unwrap();
    let mdbook_bib_prepro = Bibliography;
    md.with_preprocessor(mdbook_bib_prepro);

    match md.build() {
        Ok(_) => (),
        Err(err) => panic!("Error building CSL Nature test book: {err:?}"),
    }

    // Nature uses superscript - verify <sup> tags in output
    let mut output_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    output_path.push("test_book_csl_nature/book/biology.html");

    if output_path.exists() {
        match find_str_in_file("<sup>", output_path) {
            Ok(_) => (),
            Err(_) => panic!("Superscript tags not found in Nature style output"),
        }
    }
}
