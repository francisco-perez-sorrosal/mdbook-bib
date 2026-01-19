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
use crate::config::CitationSyntax;
use indexmap::IndexMap;
use mdbook_preprocessor::book::Chapter;
use rstest::rstest;
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
        &CitationSyntax::Default,
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
        &CitationSyntax::Default,
    );
    assert!(text_with_citations.contains(">fps</a>"));
    assert!(text_with_citations.contains("[Unknown bib ref:"));
}

#[rstest]
#[case::root_chapter("source.md", "bibliography.html")]
#[case::one_level_down("dir1/source.md", "../bibliography.html")]
#[case::two_levels_down("dir1/dir2/source.md", "../../bibliography.html")]
#[case::non_canonical_path("dir1/dir2/../source.md", "../bibliography.html")]
fn citations_in_subfolders_link_properly(#[case] chapter_path: &str, #[case] expected_link: &str) {
    let mut bibliography = dummy_bibliography();
    let backend = create_citation_backend();

    let chapter = Chapter::new(
        "",
        DUMMY_TEXT_WITH_2_VALID_CITE_PLACEHOLDERS.into(),
        chapter_path,
        vec![],
    );

    let mut last_index = 0;
    let text_with_citations = crate::citation::replace_all_placeholders(
        &chapter,
        &mut bibliography,
        &mut HashSet::new(),
        &backend,
        &mut last_index,
        &CitationSyntax::Default,
    );

    assert!(
        text_with_citations.contains(&format!(r#"href="{expected_link}#fps""#)),
        "Expecting link to '{expected_link}' in '{text_with_citations}'",
    );
    assert!(
        text_with_citations.contains(&format!(r#"href="{expected_link}#rust_book""#)),
        "Expecting link to '{expected_link}' in '{text_with_citations}'",
    );
}

#[test]
fn citations_in_draft_chapter_link_properly() {
    let mut bibliography = dummy_bibliography();
    let backend = create_citation_backend();

    let mut draft_chapter = Chapter::new_draft("", vec![]);
    draft_chapter.content = DUMMY_TEXT_WITH_2_VALID_CITE_PLACEHOLDERS.into();

    let mut last_index = 0;
    let text_with_citations = crate::citation::replace_all_placeholders(
        &draft_chapter,
        &mut bibliography,
        &mut HashSet::new(),
        &backend,
        &mut last_index,
        &CitationSyntax::Default,
    );

    assert!(text_with_citations.contains(r#"href="bibliography.html#fps""#));
    assert!(text_with_citations.contains(r#"href="bibliography.html#rust_book""#));
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
        &CitationSyntax::Default,
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
        &CitationSyntax::Default,
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
// Regex Pattern Tests (Parametrized)
// =============================================================================

#[rstest]
#[case::simple_key("{{#cite mdBook}}", "mdBook")]
#[case::key_with_colon("{{#cite DUMMY:1}}", "DUMMY:1")]
#[case::key_with_hyphen("{{#cite test-key}}", "test-key")]
#[case::key_with_underscore("{{#cite test_key}}", "test_key")]
#[case::doi_format("{{#cite 10.1145/3508461}}", "10.1145/3508461")]
fn test_ref_pattern_captures_key(#[case] input: &str, #[case] expected_key: &str) {
    use crate::REF_PATTERN;
    use regex::Regex;

    let re = Regex::new(REF_PATTERN).unwrap();
    let captures = re
        .captures(input)
        .unwrap_or_else(|| panic!("Should match: {input}"));
    let captured_key = captures.get(1).expect("Should capture key").as_str().trim();
    assert_eq!(captured_key, expected_key);
}

#[rstest]
#[case::doi_with_dots("@@10.1145/3508461", "10.1145/3508461")]
#[case::simple_key("@@simple_key", "simple_key")]
#[case::key_with_dots("@@key.with.dots", "key.with.dots")]
#[case::key_with_dashes("@@key-with-dashes", "key-with-dashes")]
#[case::key_with_underscores("@@key_with_underscores", "key_with_underscores")]
fn test_at_ref_pattern_captures_key(#[case] input: &str, #[case] expected_key: &str) {
    use crate::AT_REF_PATTERN;
    use regex::Regex;

    let re = Regex::new(AT_REF_PATTERN).unwrap();
    let captures = re
        .captures(input)
        .unwrap_or_else(|| panic!("Should match: {input}"));
    let captured_key = captures.get(2).expect("Should capture key").as_str();
    assert_eq!(captured_key, expected_key);
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
        &CitationSyntax::Default,
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
        &CitationSyntax::Default,
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

#[rstest]
#[case::include("{{#include file.rs}}")]
#[case::title("{{#title My Custom Title}}")]
#[case::playground("{{#playground example.rs}}")]
#[case::rustdoc_include("{{#rustdoc_include file.rs:2}}")]
#[case::include_with_range("{{#include file.rs:2:10}}")]
fn test_ref_pattern_excludes_mdbook_expressions(#[case] input: &str) {
    use crate::REF_PATTERN;
    use regex::Regex;

    let re = Regex::new(REF_PATTERN).unwrap();
    assert!(
        !re.is_match(input),
        "Pattern should NOT match mdBook expression: {input}"
    );
}

// =============================================================================
// Pandoc Citation Syntax Tests
// =============================================================================

#[test]
fn test_pandoc_author_in_text_citation() {
    use crate::citation::PANDOC_CITE_PATTERN;
    use regex::Regex;

    let re = Regex::new(PANDOC_CITE_PATTERN).unwrap();

    // Should match @key at word boundaries
    // Pattern captures: group 1 = prefix (empty or char), group 2 = key
    let cases = vec![
        ("@Smith2024", "Smith2024"),
        ("See @Jones2020 for details", "Jones2020"),
        ("As @Author_Name argues", "Author_Name"),
    ];

    for (input, expected_key) in cases {
        let caps = re.captures(input);
        assert!(caps.is_some(), "Should match: {input}");
        let key = caps.unwrap().get(2).unwrap().as_str();
        assert_eq!(key, expected_key, "Should capture key from: {input}");
    }
}

#[test]
fn test_pandoc_pattern_does_not_match_emails() {
    use crate::citation::PANDOC_CITE_PATTERN;
    use regex::Regex;

    let re = Regex::new(PANDOC_CITE_PATTERN).unwrap();

    // Email addresses should NOT be matched
    // The pattern requires a non-word char before @, so "user@..." won't match
    let non_matches = vec![
        "user@example.com",
        "email@domain.org",
        "test@email.co.uk",
        "name@company.io",
    ];

    for input in non_matches {
        assert!(!re.is_match(input), "Should NOT match email: {input}");
    }
}

#[test]
fn test_pandoc_pattern_does_not_match_url_mentions() {
    use crate::citation::PANDOC_CITE_PATTERN;
    use regex::Regex;

    let re = Regex::new(PANDOC_CITE_PATTERN).unwrap();

    // URLs with @ mentions should NOT be matched
    // The pattern excludes / before @ to handle URL paths
    let non_matches = vec![
        "https://twitter.com/@rustlang",
        "https://github.com/@user",
        "http://example.com/@mention",
        "git@github.com:user/repo.git", // Also git URLs
    ];

    for input in non_matches {
        assert!(!re.is_match(input), "Should NOT match URL mention: {input}");
    }
}

#[test]
fn test_pandoc_pattern_does_not_match_double_at() {
    use crate::citation::PANDOC_CITE_PATTERN;
    use regex::Regex;

    let re = Regex::new(PANDOC_CITE_PATTERN).unwrap();

    // @@key should NOT be matched by Pandoc single-@ pattern
    // The pattern requires non-@ char before @, so @@ won't match
    let non_matches = vec!["@@Smith2024", "@@key"];

    for input in non_matches {
        assert!(!re.is_match(input), "Should NOT match double-at: {input}");
    }

    // "text @@citation" - the @ before @citation excludes matching @citation
    let input = "text @@citation more";
    assert!(
        !re.is_match(input),
        "Should NOT match @citation after @@ prefix"
    );
}

#[test]
fn test_pandoc_bracketed_citation() {
    use crate::citation::PANDOC_BRACKETED_PATTERN;
    use regex::Regex;

    let re = Regex::new(PANDOC_BRACKETED_PATTERN).unwrap();

    let cases = vec![
        ("[@Smith2024]", "Smith2024"),
        ("see [@Jones2020]", "Jones2020"),
        ("documented [@Author_Name].", "Author_Name"),
    ];

    for (input, expected_key) in cases {
        let caps = re.captures(input);
        assert!(caps.is_some(), "Should match: {input}");
        let key = caps.unwrap().get(1).unwrap().as_str();
        assert_eq!(key, expected_key, "Should capture key from: {input}");
    }
}

#[test]
fn test_pandoc_suppress_author_citation() {
    use crate::citation::PANDOC_SUPPRESS_AUTHOR_PATTERN;
    use regex::Regex;

    let re = Regex::new(PANDOC_SUPPRESS_AUTHOR_PATTERN).unwrap();

    let cases = vec![
        ("[-@Smith2024]", "Smith2024"),
        ("Smith argues [-@Smith2024]", "Smith2024"),
        ("As noted [-@Author_Name].", "Author_Name"),
    ];

    for (input, expected_key) in cases {
        let caps = re.captures(input);
        assert!(caps.is_some(), "Should match: {input}");
        let key = caps.unwrap().get(1).unwrap().as_str();
        assert_eq!(key, expected_key, "Should capture key from: {input}");
    }
}

#[test]
fn test_pandoc_bracketed_does_not_match_markdown_links() {
    use crate::citation::PANDOC_BRACKETED_PATTERN;
    use regex::Regex;

    let re = Regex::new(PANDOC_BRACKETED_PATTERN).unwrap();

    // Markdown links [text](url) should NOT be matched
    let non_matches = vec![
        "[link text](https://example.com)",
        "[Documentation](../docs/index.html)",
        "[See here](./local.md)",
    ];

    for input in non_matches {
        assert!(
            !re.is_match(input),
            "Should NOT match markdown link: {input}"
        );
    }
}

#[test]
fn test_pandoc_syntax_full_replacement() {
    let content = r#"
According to @Smith2024, this is important.
This has been documented [@Jones2020].
Smith argues [-@Smith2024] that...
Legacy syntax still works: @@legacy_key and {{#cite another_key}}.
"#;

    let mut bibliography = IndexMap::new();
    for key in ["Smith2024", "Jones2020", "legacy_key", "another_key"] {
        bibliography.insert(
            key.to_string(),
            BibItemBuilder::article(key)
                .title(&format!("Title for {key}"))
                .authors(&["Author"])
                .year("2024")
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
        &CitationSyntax::Pandoc,
    );

    // All citations should be found
    assert!(cited.contains("Smith2024"), "Should find Smith2024");
    assert!(cited.contains("Jones2020"), "Should find Jones2020");
    assert!(cited.contains("legacy_key"), "Should find legacy_key");
    assert!(cited.contains("another_key"), "Should find another_key");

    // Original patterns should be gone
    assert!(!result.contains("@Smith2024"), "Should replace @Smith2024");
    assert!(
        !result.contains("[@Jones2020]"),
        "Should replace [@Jones2020]"
    );
    assert!(
        !result.contains("[-@Smith2024]"),
        "Should replace [-@Smith2024]"
    );
    assert!(
        !result.contains("@@legacy_key"),
        "Should replace @@legacy_key"
    );
    assert!(
        !result.contains("{{#cite another_key}}"),
        "Should replace {{#cite}}"
    );
}

#[test]
fn test_code_block_protection() {
    let content = r#"
Regular citation @Smith2024 should be replaced.

```rust
// This @NotACitation should NOT be replaced
let x = 1;
```

Inline code `@AlsoNotACitation` should be protected.

~~~python
# @AnotherNonCitation in fenced block
print("hello")
~~~

Back to regular text with @Jones2020.
"#;

    let mut bibliography = IndexMap::new();
    for key in [
        "Smith2024",
        "Jones2020",
        "NotACitation",
        "AlsoNotACitation",
        "AnotherNonCitation",
    ] {
        bibliography.insert(
            key.to_string(),
            BibItemBuilder::article(key)
                .title(&format!("Title for {key}"))
                .authors(&["Author"])
                .year("2024")
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
    let backend = create_citation_backend_with_template("[CITE:{{item.citation_key}}]");
    let mut last_index = 0;

    let result = crate::citation::replace_all_placeholders(
        &chapter,
        &mut bibliography,
        &mut cited,
        &backend,
        &mut last_index,
        &CitationSyntax::Pandoc,
    );

    // Only regular citations should be found
    assert!(cited.contains("Smith2024"), "Should find Smith2024");
    assert!(cited.contains("Jones2020"), "Should find Jones2020");

    // Code block citations should NOT be found
    assert!(
        !cited.contains("NotACitation"),
        "Should NOT find citation in code block"
    );
    assert!(
        !cited.contains("AlsoNotACitation"),
        "Should NOT find citation in inline code"
    );
    assert!(
        !cited.contains("AnotherNonCitation"),
        "Should NOT find citation in ~~~ block"
    );

    // Code blocks should be preserved with original @ symbols
    assert!(
        result.contains("@NotACitation"),
        "Code block content should be preserved"
    );
    assert!(
        result.contains("`@AlsoNotACitation`"),
        "Inline code should be preserved"
    );
    assert!(
        result.contains("@AnotherNonCitation"),
        "Fenced block content should be preserved"
    );

    // Regular citations should be replaced
    assert!(
        result.contains("[CITE:Smith2024]"),
        "Regular citation should be replaced"
    );
    assert!(
        result.contains("[CITE:Jones2020]"),
        "Regular citation should be replaced"
    );
}

#[test]
fn test_escaped_at_symbol() {
    let content = r#"
Contact me at user\@example.com for more info.
Citation @Smith2024 should still work.
Another escaped: admin\@server.local
"#;

    let mut bibliography = IndexMap::new();
    bibliography.insert(
        "Smith2024".to_string(),
        BibItemBuilder::article("Smith2024")
            .title("Test Paper")
            .authors(&["Smith"])
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
    let backend = create_citation_backend_with_template("[CITE:{{item.citation_key}}]");
    let mut last_index = 0;

    let result = crate::citation::replace_all_placeholders(
        &chapter,
        &mut bibliography,
        &mut cited,
        &backend,
        &mut last_index,
        &CitationSyntax::Pandoc,
    );

    // Citation should be found and replaced
    assert!(cited.contains("Smith2024"), "Should find Smith2024");
    assert!(
        result.contains("[CITE:Smith2024]"),
        "Should replace citation"
    );

    // Escaped @ should become literal @
    assert!(
        result.contains("user@example.com"),
        "Escaped @ should become literal: {result}"
    );
    assert!(
        result.contains("admin@server.local"),
        "Escaped @ should become literal"
    );

    // No \@ should remain
    assert!(
        !result.contains(r"\@"),
        "No escaped @ patterns should remain"
    );
}

#[test]
fn test_pandoc_syntax_disabled_by_default() {
    let content = r#"
This @citation should NOT be replaced when Pandoc syntax is disabled.
But @@legacy_key should still work.
And {{#cite another_key}} too.
"#;

    let mut bibliography = IndexMap::new();
    for key in ["citation", "legacy_key", "another_key"] {
        bibliography.insert(
            key.to_string(),
            BibItemBuilder::article(key)
                .title(&format!("Title for {key}"))
                .authors(&["Author"])
                .year("2024")
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
    let backend = create_citation_backend_with_template("[{{item.citation_key}}]");
    let mut last_index = 0;

    let result = crate::citation::replace_all_placeholders(
        &chapter,
        &mut bibliography,
        &mut cited,
        &backend,
        &mut last_index,
        &CitationSyntax::Default, // Default syntax, Pandoc disabled
    );

    // @citation should NOT be replaced (Pandoc syntax disabled)
    assert!(
        !cited.contains("citation"),
        "Pandoc @citation should NOT be processed in default mode"
    );
    assert!(
        result.contains("@citation"),
        "@citation should remain unchanged"
    );

    // Legacy syntax should still work
    assert!(cited.contains("legacy_key"), "@@legacy_key should work");
    assert!(cited.contains("another_key"), "{{#cite}} should work");
}
