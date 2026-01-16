//! Tests for citation replacement and regex pattern matching.
//!
//! This module covers:
//! - Citation placeholder replacement (`{{#cite key}}` and `@@key`)
//! - Regex pattern matching
//! - Subfolder linking
//! - BibLaTeX-compliant citation keys

use super::common::{
    create_citation_backend, create_citation_backend_with_template, dummy_bibliography,
    BibItemBuilder, DUMMY_TEXT_WITH_2_VALID_CITE_PLACEHOLDERS,
    DUMMY_TEXT_WITH_A_VALID_AND_AN_INVALID_CITE_PLACEHOLDERS,
};
use indexmap::IndexMap;
use mdbook_preprocessor::book::Chapter;
use std::collections::HashSet;

// =============================================================================
// Citation Replacement Tests
// =============================================================================

#[test]
fn valid_and_invalid_citations_are_replaced_properly_in_book_text() {
    let mut bibliography = dummy_bibliography();
    let mut cited: HashSet<String> = HashSet::new();

    // Check valid references included in a dummy text
    let chapter = Chapter::new(
        "",
        DUMMY_TEXT_WITH_2_VALID_CITE_PLACEHOLDERS.into(),
        "source.md",
        vec![],
    );

    let backend = create_citation_backend();
    let mut last_index = 0;
    let text_with_citations = crate::citation::replace_all_placeholders(
        &chapter,
        &mut bibliography,
        &mut cited,
        &backend,
        &mut last_index,
    );

    assert!(text_with_citations.contains(r#"href="bibliography.html#fps""#));
    assert!(text_with_citations.contains(r#"href="bibliography.html#rust_book""#));

    // Check a mix of valid and invalid references
    let chapter = Chapter::new(
        "",
        DUMMY_TEXT_WITH_A_VALID_AND_AN_INVALID_CITE_PLACEHOLDERS.into(),
        "source.md",
        vec![],
    );
    let mut last_index = 0;
    let text_with_citations = crate::citation::replace_all_placeholders(
        &chapter,
        &mut bibliography,
        &mut cited,
        &backend,
        &mut last_index,
    );
    assert!(text_with_citations.contains(">fps</a>"));
    assert!(text_with_citations.contains("[Unknown bib ref:"));
}

#[test]
fn citations_in_subfolders_link_properly() {
    let mut bibliography = dummy_bibliography();
    let backend = create_citation_backend();

    let mut check_citations_for = |chapter: &Chapter, link: &str| {
        let mut last_index = 0;
        let text_with_citations = crate::citation::replace_all_placeholders(
            chapter,
            &mut bibliography,
            &mut HashSet::new(),
            &backend,
            &mut last_index,
        );

        assert!(
            text_with_citations.contains(&format!(r#"href="{link}#fps""#)),
            "Expecting link to '{link}' in string '{text_with_citations}'",
        );
        assert!(
            text_with_citations.contains(&format!(r#"href="{link}#rust_book""#)),
            "Expecting link to '{link}' in string '{text_with_citations}'",
        );
    };

    let mut draft_chapter = Chapter::new_draft("", vec![]);
    draft_chapter.content = DUMMY_TEXT_WITH_2_VALID_CITE_PLACEHOLDERS.into();
    check_citations_for(&draft_chapter, "bibliography.html");

    let chapter_root = Chapter::new(
        "",
        DUMMY_TEXT_WITH_2_VALID_CITE_PLACEHOLDERS.into(),
        "source.md",
        vec![],
    );
    check_citations_for(&chapter_root, "bibliography.html");

    let chapter_1down = Chapter::new(
        "",
        DUMMY_TEXT_WITH_2_VALID_CITE_PLACEHOLDERS.into(),
        "dir1/source.md",
        vec![],
    );
    check_citations_for(&chapter_1down, "../bibliography.html");

    let chapter_2down = Chapter::new(
        "",
        DUMMY_TEXT_WITH_2_VALID_CITE_PLACEHOLDERS.into(),
        "dir1/dir2/source.md",
        vec![],
    );
    check_citations_for(&chapter_2down, "../../bibliography.html");

    let chapter_noncanon = Chapter::new(
        "",
        DUMMY_TEXT_WITH_2_VALID_CITE_PLACEHOLDERS.into(),
        "dir1/dir2/../source.md",
        vec![],
    );
    check_citations_for(&chapter_noncanon, "../bibliography.html");
}

#[test]
fn debug_replace_all_placeholders() {
    let content = r#"
This is a reference {{#cite mdBook}} that has to be resolved to the right bibliography file.
This is a reference to a non-existing book that reports a bug @@mdBookWithAuthorsWithANDInTheirName that was resolved.
This is a reference to {{#cite DUMMY:1}}
"#;

    let mut bibliography = IndexMap::new();
    bibliography.insert(
        "mdBook".to_string(),
        BibItemBuilder::misc("mdBook")
            .title("mdBook Documentation")
            .authors(&["Various Contributors"])
            .year("2015")
            .summary("mdBook is a command line tool.")
            .url("https://rust-lang.github.io/mdBook/")
            .build(),
    );
    bibliography.insert(
        "mdBookWithAuthorsWithANDInTheirName".to_string(),
        BibItemBuilder::misc("mdBookWithAuthorsWithANDInTheirName")
            .title("Bug when rendering authors that include the `and` substring in their names")
            .authors_parts(vec![vec![
                "Jane A. Doeander".to_string(),
                "John B. Doeanderson".to_string(),
            ]])
            .year("2023")
            .summary("What a book about nothing...")
            .url("https://github.com/francisco-perez-sorrosal/mdbook-bib/issues/44")
            .build(),
    );
    bibliography.insert(
        "DUMMY:1".to_string(),
        BibItemBuilder::misc("DUMMY:1")
            .title("The Book without Title")
            .authors_parts(vec![vec!["John".to_string(), "Doe".to_string()]])
            .year("2100")
            .summary("N/A")
            .build(),
    );

    let chapter = Chapter::new(
        "Test",
        content.to_string(),
        std::path::PathBuf::new(),
        vec![],
    );
    let mut cited = HashSet::new();
    let backend = create_citation_backend_with_template("{{item.citation_key}}");
    let mut last_index = 0;
    let _ = crate::citation::replace_all_placeholders(
        &chapter,
        &mut bibliography,
        &mut cited,
        &backend,
        &mut last_index,
    );
}

#[test]
fn test_citation_with_dots_replacement() {
    let content = r#"
This is a reference to a paper with DOI @@10.1145/3508461 that should be properly resolved.
This is another reference @@simple_key that should also work.
"#;

    let mut bibliography = IndexMap::new();
    bibliography.insert(
        "10.1145/3508461".to_string(),
        BibItemBuilder::article("10.1145/3508461")
            .title("Some Paper with DOI")
            .authors(&["Author Name"])
            .year("2023")
            .summary("A paper with a DOI citation key")
            .url("https://doi.org/10.1145/3508461")
            .build(),
    );
    bibliography.insert(
        "simple_key".to_string(),
        BibItemBuilder::article("simple_key")
            .title("Simple Paper")
            .authors(&["Another Author"])
            .year("2023")
            .summary("A paper with a simple citation key")
            .build(),
    );

    let chapter = Chapter::new(
        "Test",
        content.to_string(),
        std::path::PathBuf::new(),
        vec![],
    );
    let mut cited = HashSet::new();
    let backend = create_citation_backend_with_template("{{item.citation_key}}");
    let mut last_index = 0;

    let result = crate::citation::replace_all_placeholders(
        &chapter,
        &mut bibliography,
        &mut cited,
        &backend,
        &mut last_index,
    );

    // Check that both citations were found and added to cited set
    assert!(cited.contains("10.1145/3508461"));
    assert!(cited.contains("simple_key"));

    // Check that the replacements were made correctly
    assert!(result.contains("10.1145/3508461"));
    assert!(result.contains("simple_key"));
    assert!(!result.contains("@@10.1145/3508461"));
    assert!(!result.contains("@@simple_key"));
}

// =============================================================================
// Regex Pattern Tests
// =============================================================================

#[test]
fn test_regex_pattern() {
    use crate::REF_PATTERN;
    use regex::Regex;

    let re = Regex::new(REF_PATTERN).unwrap();

    let test_cases = vec![
        ("{{#cite mdBook}}", "mdBook"),
        ("{{#cite DUMMY:1}}", "DUMMY:1"),
        ("{{#cite test-key}}", "test-key"),
        ("{{#cite test_key}}", "test_key"),
    ];

    for (test_case, expected_key) in test_cases {
        println!("Testing: '{test_case}'");
        if let Some(captures) = re.captures(test_case) {
            println!("  Match found!");
            println!("  Full match: '{}'", captures.get(0).unwrap().as_str());
            if let Some(cite_key) = captures.get(1) {
                let key = cite_key.as_str().trim();
                println!("  Citation key: '{key}'");
                assert_eq!(
                    key, expected_key,
                    "Citation key should match for: {test_case}"
                );
            } else {
                panic!("No citation key captured for: {test_case}");
            }
        } else {
            panic!("Pattern should match citation: {test_case}");
        }
        println!();
    }
}

#[test]
fn test_at_ref_pattern_with_dots() {
    use crate::AT_REF_PATTERN;
    use regex::Regex;

    let re = Regex::new(AT_REF_PATTERN).unwrap();

    let test_cases = vec![
        "@@10.1145/3508461",
        "@@simple_key",
        "@@key.with.dots",
        "@@key-with-dashes",
        "@@key_with_underscores",
    ];

    for test_case in test_cases {
        if let Some(captures) = re.captures(test_case) {
            if let Some(cite_key) = captures.get(2) {
                // Verify that citation keys with dots are captured correctly
                if test_case.contains("10.1145/3508461") {
                    assert_eq!(cite_key.as_str(), "10.1145/3508461");
                }
            }
        } else {
            panic!("No match found for test case: {test_case}");
        }
    }
}

#[test]
fn test_at_ref_followed_by_punctuation() {
    let content = r#"
This book is written in Rust @@Klabnik2018.
Another citation at end of sentence @@fps!
What about questions @@simple_key?
Or maybe colons @@another_key: it should work.
We reference this paper @@10.1145/3508461.
See @@ref1, @@ref2, and @@ref3.
Citations in parentheses (see @@Jones2019).
"#;

    let mut bibliography = IndexMap::new();
    let keys = [
        ("Klabnik2018", "The Rust Programming Language", "2018"),
        ("fps", "Test Entry", "2020"),
        ("simple_key", "Simple Paper", "2023"),
        ("another_key", "Another Paper", "2024"),
        ("10.1145/3508461", "Paper with DOI", "2023"),
        ("ref1", "First Reference", "2020"),
        ("ref2", "Second Reference", "2021"),
        ("ref3", "Third Reference", "2022"),
        ("Jones2019", "Jones Paper", "2019"),
    ];

    for (key, title, year) in keys {
        bibliography.insert(
            key.to_string(),
            BibItemBuilder::article(key)
                .title(title)
                .authors(&["Author"])
                .year(year)
                .summary("Test")
                .build(),
        );
    }

    let chapter = Chapter::new(
        "Test",
        content.to_string(),
        std::path::PathBuf::new(),
        vec![],
    );
    let mut cited = HashSet::new();
    let backend = create_citation_backend_with_template("{{item.citation_key}}");
    let mut last_index = 0;

    let result = crate::citation::replace_all_placeholders(
        &chapter,
        &mut bibliography,
        &mut cited,
        &backend,
        &mut last_index,
    );

    // Check that all citations were found
    for (key, _, _) in &keys {
        assert!(cited.contains(*key), "Should contain {key}");
        assert!(result.contains(*key), "Result should contain {key}");
        assert!(
            !result.contains(&format!("@@{key}")),
            "Original @@ pattern should be gone for {key}"
        );
    }

    // Check that punctuation is preserved after the citation
    assert!(result.contains("Klabnik2018."));
    assert!(result.contains("fps!"));
    assert!(result.contains("simple_key?"));
    assert!(result.contains("another_key:"));
    assert!(result.contains("10.1145/3508461."));

    // Check multiple citations with commas are handled correctly
    assert!(result.contains("ref1,"));
    assert!(result.contains("ref2,"));
    assert!(result.contains("ref3."));

    // Check citation in parentheses is handled correctly
    assert!(result.contains("Jones2019)"));
}

#[test]
fn test_biblatex_compliant_citation_keys() {
    use crate::{AT_REF_PATTERN, REF_PATTERN};
    use regex::Regex;

    let ref_re = Regex::new(REF_PATTERN).unwrap();
    let at_re = Regex::new(AT_REF_PATTERN).unwrap();

    // BibLaTeX-compliant citation keys with various safe characters
    let keys = vec![
        "Klabnik2018",
        "10.1145/3508461",           // DOI with dots and slash
        "doi:10.5555/12345",         // DOI with colon prefix
        "arXiv:2301.12345",          // arXiv with colon
        "key_with_underscores",      // underscores
        "key-with-hyphens",          // hyphens
        "user@domain",               // at symbol
        "mixed:key_2023.1-final@v2", // combination
    ];

    // Test REF_PATTERN ({{#cite key}})
    for key in &keys {
        let text = format!("{{{{#cite {key}}}}}");
        assert!(
            ref_re.is_match(&text),
            "REF_PATTERN should match BibLaTeX-compliant key: {key}"
        );
        if let Some(caps) = ref_re.captures(&text) {
            if let Some(captured) = caps.get(1) {
                assert_eq!(
                    captured.as_str(),
                    *key,
                    "REF_PATTERN should capture the full key"
                );
            }
        }
    }

    // Test AT_REF_PATTERN (@@ syntax) with trailing punctuation
    for key in &keys {
        let text = format!("See @@{key}.");
        assert!(
            at_re.is_match(&text),
            "AT_REF_PATTERN should match BibLaTeX-compliant key: {key}"
        );
        if let Some(caps) = at_re.captures(&text) {
            if let Some(captured) = caps.get(2) {
                assert_eq!(
                    captured.as_str(),
                    *key,
                    "AT_REF_PATTERN should capture the full key without trailing punctuation"
                );
            }
        }
    }

    // Test with actual replacement to ensure keys work end-to-end
    let content = r#"
Test {{#cite doi:10.5555/12345}} and @@arXiv:2301.12345.
User citation @@user@domain is valid.
"#;

    let mut bibliography = IndexMap::new();
    bibliography.insert(
        "doi:10.5555/12345".to_string(),
        BibItemBuilder::article("doi:10.5555/12345")
            .title("DOI Paper")
            .authors(&["Author"])
            .year("2023")
            .url("https://doi.org/10.5555/12345")
            .build(),
    );
    bibliography.insert(
        "arXiv:2301.12345".to_string(),
        BibItemBuilder::article("arXiv:2301.12345")
            .title("arXiv Paper")
            .authors(&["Researcher"])
            .year("2023")
            .url("https://arxiv.org/abs/2301.12345")
            .build(),
    );
    bibliography.insert(
        "user@domain".to_string(),
        BibItemBuilder::misc("user@domain")
            .title("User Citation")
            .authors(&["User"])
            .year("2024")
            .build(),
    );

    let chapter = Chapter::new(
        "Test",
        content.to_string(),
        std::path::PathBuf::new(),
        vec![],
    );
    let mut cited = HashSet::new();
    let backend = create_citation_backend_with_template("{{item.citation_key}}");
    let mut last_index = 0;

    let result = crate::citation::replace_all_placeholders(
        &chapter,
        &mut bibliography,
        &mut cited,
        &backend,
        &mut last_index,
    );

    // Verify all citations were found
    assert!(cited.contains("doi:10.5555/12345"));
    assert!(cited.contains("arXiv:2301.12345"));
    assert!(cited.contains("user@domain"));

    // Verify replacements occurred
    assert!(result.contains("doi:10.5555/12345"));
    assert!(result.contains("arXiv:2301.12345"));
    assert!(result.contains("user@domain"));

    // Verify original patterns are gone
    assert!(!result.contains("{{#cite doi:10.5555/12345}}"));
    assert!(!result.contains("@@arXiv:2301.12345"));
    assert!(!result.contains("@@user@domain"));
}

#[test]
fn test_ref_pattern_excludes_mdbook_expressions() {
    use crate::REF_PATTERN;
    use regex::Regex;

    let re = Regex::new(REF_PATTERN).unwrap();

    // These should NOT match (mdBook expressions)
    let should_not_match = vec![
        "{{#include file.rs}}",
        "{{#title My Custom Title}}",
        "{{#playground example.rs}}",
        "{{#rustdoc_include file.rs:2}}",
        "{{#include file.rs:2:10}}",
    ];

    for test_case in should_not_match {
        assert!(
            !re.is_match(test_case),
            "Pattern should NOT match mdBook expression: {test_case}"
        );
    }

    // These SHOULD match (citation expressions)
    let should_match = vec![
        ("{{#cite mdBook}}", "mdBook"),
        ("{{#cite DUMMY:1}}", "DUMMY:1"),
        ("{{#cite test-key}}", "test-key"),
        ("{{#cite 10.1145/3508461}}", "10.1145/3508461"),
    ];

    for (test_case, expected_key) in should_match {
        if let Some(captures) = re.captures(test_case) {
            if let Some(cite_key) = captures.get(1) {
                assert_eq!(
                    cite_key.as_str().trim(),
                    expected_key,
                    "Citation key should match for: {test_case}"
                );
            } else {
                panic!("No citation key captured for: {test_case}");
            }
        } else {
            panic!("Pattern should match citation: {test_case}");
        }
    }
}
