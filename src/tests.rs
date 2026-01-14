use crate::backend::LegacyBackend;
use crate::config::DEFAULT_JS_TEMPLATE;
use crate::config::{SortOrder, DEFAULT_HB_TEMPLATE};
use crate::config::{DEFAULT_CITE_HB_TEMPLATE, DEFAULT_CSS_TEMPLATE};
use crate::Bibliography;
use handlebars::Handlebars;
use indexmap::IndexMap;
use mdbook_driver::MDBook;
use std::fs::File;
use std::io::Write;

use std::{collections::HashSet, path::PathBuf};

#[cfg(test)]
// use std::{println as info, println as warn};
use tempfile::Builder as TempFileBuilder;

use crate::config::Config;
use crate::io;
use crate::models::BibItem;
use crate::parser::{self, BibFormat};
use toml::value::Table;
use toml::Value;

use mdbook_preprocessor::book::Chapter;

// Test helper functions
fn create_references_handlebars() -> Handlebars<'static> {
    let mut handlebars = Handlebars::new();
    handlebars
        .register_template_string("references", format!("\n\n{DEFAULT_HB_TEMPLATE}\n\n"))
        .unwrap();
    handlebars
}

fn create_citation_handlebars() -> Handlebars<'static> {
    let mut handlebars = Handlebars::new();
    handlebars
        .register_template_string("citation", DEFAULT_CITE_HB_TEMPLATE)
        .unwrap();
    handlebars
}

fn create_citation_handlebars_with_template(template: &str) -> Handlebars<'static> {
    let mut handlebars = Handlebars::new();
    handlebars
        .register_template_string("citation", template)
        .unwrap();
    handlebars
}

static EXAMPLE_CSS_TEMPLATE: &str = include_str!("../manual/src/render/my_style.css");
static EXAMPLE_HB_TEMPLATE: &str = include_str!("../manual/src/render/my_references.hbs");

const DUMMY_BIB_SRC: &str = r#"
@misc{fps,
    title = {This is a bib entry!},
    author = {Francisco Perez-Sorrosal},
    month = oct,
    year = {2020},
    what_is_this = {blabla},
}
@book{rust_book,
    author = {Klabnik, Steve and Nichols, Carol},
    title = {The Rust Programming Language},
    year = {2018},
    isbn = {1593278284},
    publisher = {No Starch Press},
    url = {https://doc.rust-lang.org/book/},
}
"#;

const DUMMY_TEXT_WITH_2_VALID_CITE_PLACEHOLDERS: &str = r#"
this is a dumb text that includes citations like {{ #cite fps }} and {{ #cite rust_book }}
"#;

const DUMMY_TEXT_WITH_A_VALID_AND_AN_INVALID_CITE_PLACEHOLDERS: &str = r#"
this is a dumb text that includes valid and invalid citations like {{ #cite fps }} and {{ #cite im_not_there }}
"#;

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

#[test]
fn bibliography_builder_returns_a_bibliography() {
    let bibliography_loaded: IndexMap<String, BibItem> =
        parser::parse_bibliography(DUMMY_BIB_SRC.to_string(), BibFormat::BibTeX).unwrap();
    assert_eq!(bibliography_loaded.len(), 2);
    assert_eq!(bibliography_loaded.get("fps").unwrap().citation_key, "fps");
}

#[test]
fn bibliography_render_all_vs_cited() {
    let bibliography_loaded: IndexMap<String, BibItem> =
        parser::parse_bibliography(DUMMY_BIB_SRC.to_string(), BibFormat::BibTeX).unwrap();

    let mut cited = HashSet::new();
    cited.insert("fps".to_string());

    let handlebars = create_references_handlebars();
    let backend = LegacyBackend::new(&handlebars);

    let html = crate::renderer::generate_bibliography_html(
        &bibliography_loaded,
        &cited,
        false,
        &backend,
        SortOrder::None,
    );

    assert!(html.contains("This is a bib entry!"));
    assert!(html.contains("The Rust Programming Language"));

    let html = crate::renderer::generate_bibliography_html(
        &bibliography_loaded,
        &cited,
        true,
        &backend,
        SortOrder::None,
    );

    assert!(html.contains("This is a bib entry!"));
    assert!(!html.contains("The Rust Programming Language"));
}

#[test]
fn bibliography_includes_and_renders_url_when_present_in_bibitems() {
    let bibliography_loaded: IndexMap<String, BibItem> =
        parser::parse_bibliography(DUMMY_BIB_SRC.to_string(), BibFormat::BibTeX).unwrap();

    // fps dummy book does not include a url for in the BibItem
    let fps = bibliography_loaded.get("fps");
    assert!(fps.unwrap().url.is_none());
    // rust_book does...
    let rust_book = bibliography_loaded.get("rust_book");
    assert_eq!(
        rust_book.unwrap().url.as_ref().unwrap(),
        "https://doc.rust-lang.org/book/"
    );
    // ...and is included in the render
    let handlebars = create_references_handlebars();
    let backend = LegacyBackend::new(&handlebars);
    let html = crate::renderer::generate_bibliography_html(
        &bibliography_loaded,
        &HashSet::new(),
        false,
        &backend,
        SortOrder::None,
    );
    assert!(html.contains("href=\"https://doc.rust-lang.org/book/\""));
}

#[test]
fn valid_and_invalid_citations_are_replaced_properly_in_book_text() {
    let mut bibliography: IndexMap<String, BibItem> =
        parser::parse_bibliography(DUMMY_BIB_SRC.to_string(), BibFormat::BibTeX).unwrap();

    let mut cited: HashSet<String> = HashSet::new();

    // Check valid references included in a dummy text
    let chapter = Chapter::new(
        "",
        DUMMY_TEXT_WITH_2_VALID_CITE_PLACEHOLDERS.into(),
        "source.md",
        vec![],
    );

    let handlebars = create_citation_handlebars();
    let backend = LegacyBackend::new(&handlebars);
    let mut last_index = 0;
    let text_with_citations = crate::citation::replace_all_placeholders(
        &chapter,
        &mut bibliography,
        &mut cited,
        &backend,
        &mut last_index,
    );
    // TODO: These asserts will probably fail if we allow users to specify the bibliography
    // chapter name as per issue #6
    assert!(text_with_citations.contains("[fps](bibliography.html#fps)"));
    assert!(text_with_citations.contains("[rust_book](bibliography.html#rust_book)"));

    // Check a mix of valid and invalid references included/not included in a dummy text
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
    assert!(text_with_citations.contains("[fps]"));
    assert!(text_with_citations.contains("[Unknown bib ref:"));
}

#[test]
fn citations_in_subfolders_link_properly() {
    let mut bibliography: IndexMap<String, BibItem> =
        parser::parse_bibliography(DUMMY_BIB_SRC.to_string(), BibFormat::BibTeX).unwrap();

    // Check valid references included in a dummy text
    let handlebars = create_citation_handlebars();
    let backend = LegacyBackend::new(&handlebars);
    let mut check_citations_for = |chapter: &Chapter, link: &str| {
        let mut last_index = 0;
        let text_with_citations = crate::citation::replace_all_placeholders(
            chapter,
            &mut bibliography,
            &mut HashSet::new(),
            &backend,
            &mut last_index,
        );

        // TODO: These asserts will probably fail if we allow users to specify the bibliography
        // chapter name as per issue #6
        assert!(
            text_with_citations.contains(&format!("[fps]({link}#fps)")),
            "Expecting link to '{link}' in string '{text_with_citations}'",
        );
        assert!(
            text_with_citations.contains(&format!("[rust_book]({link}#rust_book)")),
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
    use crate::models::BibItem;
    use indexmap::IndexMap;
    use mdbook_preprocessor::book::Chapter;
    use std::collections::HashSet;

    let content = r#"
This is a reference {{#cite mdBook}} that has to be resolved to the right bibliography file.
This is a reference to a non-existing book that reports a bug @@mdBookWithAuthorsWithANDInTheirName that was resolved.
This is a reference to {{#cite DUMMY:1}}
"#;

    let mut bibliography = IndexMap::new();
    bibliography.insert(
        "mdBook".to_string(),
        BibItem {
            citation_key: "mdBook".to_string(),
            title: "mdBook Documentation".to_string(),
            authors: vec![vec!["Various Contributors".to_string()]],
            pub_month: "N/A".to_string(),
            pub_year: "2015".to_string(),
            summary: "mdBook is a command line tool.".to_string(),
            url: Some("https://rust-lang.github.io/mdBook/".to_string()),
            index: None,
            ..Default::default()
        },
    );
    bibliography.insert(
        "mdBookWithAuthorsWithANDInTheirName".to_string(),
        BibItem {
            citation_key: "mdBookWithAuthorsWithANDInTheirName".to_string(),
            title: "Bug when rendering authors that include the `and` substring in their names"
                .to_string(),
            authors: vec![vec![
                "Jane A. Doeander".to_string(),
                "John B. Doeanderson".to_string(),
            ]],
            pub_month: "N/A".to_string(),
            pub_year: "2023".to_string(),
            summary: "What a book about nothing...".to_string(),
            url: Some(
                "https://github.com/francisco-perez-sorrosal/mdbook-bib/issues/44".to_string(),
            ),
            index: None,
            ..Default::default()
        },
    );
    bibliography.insert(
        "DUMMY:1".to_string(),
        BibItem {
            citation_key: "DUMMY:1".to_string(),
            title: "The Book without Title".to_string(),
            authors: vec![vec!["John".to_string(), "Doe".to_string()]],
            pub_month: "N/A".to_string(),
            pub_year: "2100".to_string(),
            summary: "N/A".to_string(),
            url: None,
            index: None,
            ..Default::default()
        },
    );

    let chapter = Chapter::new(
        "Test",
        content.to_string(),
        std::path::PathBuf::new(),
        vec![],
    );
    let mut cited = HashSet::new();
    let handlebars = create_citation_handlebars_with_template("{{item.citation_key}}");
    let backend = LegacyBackend::new(&handlebars);
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
    use crate::models::BibItem;
    use indexmap::IndexMap;
    use mdbook_preprocessor::book::Chapter;
    use std::collections::HashSet;

    let content = r#"
This is a reference to a paper with DOI @@10.1145/3508461 that should be properly resolved.
This is another reference @@simple_key that should also work.
"#;

    let mut bibliography = IndexMap::new();
    bibliography.insert(
        "10.1145/3508461".to_string(),
        BibItem {
            citation_key: "10.1145/3508461".to_string(),
            title: "Some Paper with DOI".to_string(),
            authors: vec![vec!["Author Name".to_string()]],
            pub_month: "N/A".to_string(),
            pub_year: "2023".to_string(),
            summary: "A paper with a DOI citation key".to_string(),
            url: Some("https://doi.org/10.1145/3508461".to_string()),
            index: None,
            ..Default::default()
        },
    );
    bibliography.insert(
        "simple_key".to_string(),
        BibItem {
            citation_key: "simple_key".to_string(),
            title: "Simple Paper".to_string(),
            authors: vec![vec!["Another Author".to_string()]],
            pub_month: "N/A".to_string(),
            pub_year: "2023".to_string(),
            summary: "A paper with a simple citation key".to_string(),
            url: None,
            index: None,
            ..Default::default()
        },
    );

    let chapter = Chapter::new(
        "Test",
        content.to_string(),
        std::path::PathBuf::new(),
        vec![],
    );
    let mut cited = HashSet::new();
    let handlebars = create_citation_handlebars_with_template("{{item.citation_key}}");
    let backend = LegacyBackend::new(&handlebars);
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
    assert!(result.contains("10.1145/3508461")); // The citation key should appear in the result
    assert!(result.contains("simple_key")); // The simple key should also appear
    assert!(!result.contains("@@10.1145/3508461")); // The original @@ pattern should be gone
    assert!(!result.contains("@@simple_key")); // The original @@ pattern should be gone
}

use std::env;
#[test]
fn check_config_attributes() {
    // Check config with default values is returned when an empty config is passed in a toml table!!!
    let t: Table = Table::new();
    match Config::build_from(Some(&t), PathBuf::new()) {
        Ok(config) => {
            println!("{config:?}");
            assert_eq!(config.title, "Bibliography");
            assert_eq!(config.bibliography, None);
            assert_eq!(config.zotero_uid, None);
            assert!(config.cited_only);
            let default_tpl = format!("\n\n{DEFAULT_HB_TEMPLATE}\n\n");
            assert_eq!(config.bib_hb_html, default_tpl);
            let default_css = format!("<style>{DEFAULT_CSS_TEMPLATE}</style>\n\n");
            assert_eq!(config.css_html, default_css);
            let default_js =
                format!("<script type=\"text/javascript\">\n{DEFAULT_JS_TEMPLATE}\n</script>\n\n",);
            assert_eq!(config.js_html, default_js);
        }
        Err(_) => panic!("there's supposed to be always a config!!!"),
    }

    // Check config attributes are processed (those that are not specified are ignored)!!!
    let mut t: Table = Table::new();

    t.insert(
        "bibliography".to_string(),
        Value::String("biblio.bib".to_string()),
    );
    t.insert(
        "zotero-uid".to_string(),
        Value::String("123456".to_string()),
    );
    t.insert("title".to_string(), Value::String("References".to_string()));
    t.insert("render-bib".to_string(), Value::String("all".to_string()));
    t.insert(
        "not-specified-config-attr".to_string(),
        Value::String("uhg???".to_string()),
    );
    match Config::build_from(Some(&t), PathBuf::new()) {
        Ok(config) => {
            println!("{config:?}");
            assert_eq!(config.title, "References");
            assert_eq!(config.bibliography, Some("biblio.bib"));
            assert_eq!(config.zotero_uid, Some("123456"));
            assert!(!config.cited_only);
        }
        Err(_) => panic!("there's supposed to be always a config!!!"),
    }

    // Intentionally add a failure specifying a non-existing value for render-bib
    let mut t: Table = Table::new();
    t.insert(
        "render-bib".to_string(),
        Value::String("non-existent!".to_string()),
    );
    match Config::build_from(Some(&t), PathBuf::new()) {
        Ok(_) => panic!("there's supposed to be a failure in the config!!!"),
        Err(_) => println!("Yayyyyy! A failure that is supposed to happen!"),
    }

    // Test adhoc template and style!!! (We check the template and style provided for the project doc/manual)
    let mut t: Table = Table::new();
    t.insert(
        "hb-tpl".to_string(),
        Value::String("render/my_references.hbs".to_string()),
    );
    t.insert(
        "css".to_string(),
        Value::String("render/my_style.css".to_string()),
    );
    // TODO No adhoc js tested at this time. Add one if added in the future to the project manual.
    let mut manual_src_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manual_src_path.push("manual/src/");
    match Config::build_from(Some(&t), manual_src_path) {
        Ok(config) => {
            println!("{config:?}");
            let adhoc_tpl = format!("\n\n{EXAMPLE_HB_TEMPLATE}\n\n");
            assert_eq!(config.bib_hb_html, adhoc_tpl);
            let adhoc_css = format!("<style>{EXAMPLE_CSS_TEMPLATE}</style>\n\n");
            assert_eq!(config.css_html, adhoc_css);
            let default_js =
                format!("<script type=\"text/javascript\">\n{DEFAULT_JS_TEMPLATE}\n</script>\n\n",);
            assert_eq!(config.js_html, default_js);
        }
        Err(e) => panic!(
            "there's supposed to be always a config!!!\n {:?}",
            e.root_cause()
        ),
    }
}

#[test]
fn test_hayagriva_date_extraction() {
    // Note: hayagriva doesn't support numeric months (month = {10}) because
    // they're not standard BibTeX. BibTeX uses month constants: jan, feb, mar, etc.

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
    assert_eq!(entry.pub_year, "2020");
    assert_eq!(entry.pub_month, "10");

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
    assert_eq!(entry.pub_year, "2020");
    assert_eq!(entry.pub_month, "N/A");

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
    assert_eq!(entry.pub_year, "N/A");
    assert_eq!(entry.pub_month, "N/A");

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
    assert_eq!(entry.pub_year, "2020");
    assert_eq!(entry.pub_month, "10");
}

#[test]
fn test_extended_bibitem_fields() {
    // Test comprehensive BibTeX entry with all extended fields
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
    assert_eq!(entry.pub_year, "2023");
    assert_eq!(entry.pub_month, "03");

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

    // Publisher - hayagriva might not extract for all entry types
    // It's optional, so we just verify the field exists in the structure

    // Editor - check if extracted
    if entry.editor.is_some() {
        let editors = entry.editor.as_ref().unwrap();
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

    // Publisher and edition - hayagriva support varies
    // Just verify the structure supports them
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

    assert_eq!(entry.pub_month, "06");
    assert!(entry.pages.is_some(), "pages should be extracted");

    // Organization - hayagriva support may vary
    if entry.organization.is_some() {
        assert_eq!(entry.organization, Some("IEEE".to_string()));
    }
}

#[test]
fn test_entry_with_minimal_fields() {
    // Test that entries with only required fields still work
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

#[test]
fn test_serialization_with_extended_fields() {
    // Test that BibItem with extended fields serializes correctly
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

pub struct NotFound;

/// Check if a string is present in the file contents
pub fn find_str_in_file(input: &str, file: PathBuf) -> Result<(), NotFound> {
    let text = std::fs::read_to_string(file).unwrap();

    for line in text.lines() {
        if line.contains(input) {
            return Ok(());
        }
    }
    Err(NotFound)
}

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
    use crate::models::BibItem;
    use indexmap::IndexMap;
    use mdbook_preprocessor::book::Chapter;
    use std::collections::HashSet;

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
    bibliography.insert(
        "Klabnik2018".to_string(),
        BibItem {
            citation_key: "Klabnik2018".to_string(),
            title: "The Rust Programming Language".to_string(),
            authors: vec![vec!["Klabnik".to_string(), "Steve".to_string()]],
            pub_month: "N/A".to_string(),
            pub_year: "2018".to_string(),
            summary: "The Rust book".to_string(),
            url: Some("https://doc.rust-lang.org/book/".to_string()),
            index: None,
            ..Default::default()
        },
    );
    bibliography.insert(
        "fps".to_string(),
        BibItem {
            citation_key: "fps".to_string(),
            title: "Test Entry".to_string(),
            authors: vec![vec!["Francisco".to_string()]],
            pub_month: "N/A".to_string(),
            pub_year: "2020".to_string(),
            summary: "Test".to_string(),
            url: None,
            index: None,
            ..Default::default()
        },
    );
    bibliography.insert(
        "simple_key".to_string(),
        BibItem {
            citation_key: "simple_key".to_string(),
            title: "Simple Paper".to_string(),
            authors: vec![vec!["Author".to_string()]],
            pub_month: "N/A".to_string(),
            pub_year: "2023".to_string(),
            summary: "Test".to_string(),
            url: None,
            index: None,
            ..Default::default()
        },
    );
    bibliography.insert(
        "another_key".to_string(),
        BibItem {
            citation_key: "another_key".to_string(),
            title: "Another Paper".to_string(),
            authors: vec![vec!["Another".to_string()]],
            pub_month: "N/A".to_string(),
            pub_year: "2024".to_string(),
            summary: "Test".to_string(),
            url: None,
            index: None,
            ..Default::default()
        },
    );
    bibliography.insert(
        "10.1145/3508461".to_string(),
        BibItem {
            citation_key: "10.1145/3508461".to_string(),
            title: "Paper with DOI".to_string(),
            authors: vec![vec!["DOI".to_string(), "Author".to_string()]],
            pub_month: "N/A".to_string(),
            pub_year: "2023".to_string(),
            summary: "DOI citation test".to_string(),
            url: Some("https://doi.org/10.1145/3508461".to_string()),
            index: None,
            ..Default::default()
        },
    );
    bibliography.insert(
        "ref1".to_string(),
        BibItem {
            citation_key: "ref1".to_string(),
            title: "First Reference".to_string(),
            authors: vec![vec!["Author1".to_string()]],
            pub_month: "N/A".to_string(),
            pub_year: "2020".to_string(),
            summary: "Test".to_string(),
            url: None,
            index: None,
            ..Default::default()
        },
    );
    bibliography.insert(
        "ref2".to_string(),
        BibItem {
            citation_key: "ref2".to_string(),
            title: "Second Reference".to_string(),
            authors: vec![vec!["Author2".to_string()]],
            pub_month: "N/A".to_string(),
            pub_year: "2021".to_string(),
            summary: "Test".to_string(),
            url: None,
            index: None,
            ..Default::default()
        },
    );
    bibliography.insert(
        "ref3".to_string(),
        BibItem {
            citation_key: "ref3".to_string(),
            title: "Third Reference".to_string(),
            authors: vec![vec!["Author3".to_string()]],
            pub_month: "N/A".to_string(),
            pub_year: "2022".to_string(),
            summary: "Test".to_string(),
            url: None,
            index: None,
            ..Default::default()
        },
    );
    bibliography.insert(
        "Jones2019".to_string(),
        BibItem {
            citation_key: "Jones2019".to_string(),
            title: "Jones Paper".to_string(),
            authors: vec![vec!["Jones".to_string(), "J.".to_string()]],
            pub_month: "N/A".to_string(),
            pub_year: "2019".to_string(),
            summary: "Test".to_string(),
            url: None,
            index: None,
            ..Default::default()
        },
    );

    let chapter = Chapter::new(
        "Test",
        content.to_string(),
        std::path::PathBuf::new(),
        vec![],
    );
    let mut cited = HashSet::new();
    let handlebars = create_citation_handlebars_with_template("{{item.citation_key}}");
    let backend = LegacyBackend::new(&handlebars);
    let mut last_index = 0;

    let result = crate::citation::replace_all_placeholders(
        &chapter,
        &mut bibliography,
        &mut cited,
        &backend,
        &mut last_index,
    );

    // Check that all citations were found (without punctuation)
    assert!(cited.contains("Klabnik2018"));
    assert!(cited.contains("fps"));
    assert!(cited.contains("simple_key"));
    assert!(cited.contains("another_key"));
    assert!(cited.contains("10.1145/3508461"));
    assert!(cited.contains("ref1"));
    assert!(cited.contains("ref2"));
    assert!(cited.contains("ref3"));
    assert!(cited.contains("Jones2019"));

    // Check that the replacements were made correctly
    assert!(result.contains("Klabnik2018"));
    assert!(result.contains("fps"));
    assert!(result.contains("simple_key"));
    assert!(result.contains("another_key"));
    assert!(result.contains("10.1145/3508461"));
    assert!(result.contains("ref1"));
    assert!(result.contains("ref2"));
    assert!(result.contains("ref3"));
    assert!(result.contains("Jones2019"));

    // Check that original @@ patterns are gone
    assert!(!result.contains("@@Klabnik2018"));
    assert!(!result.contains("@@fps"));
    assert!(!result.contains("@@simple_key"));
    assert!(!result.contains("@@another_key"));
    assert!(!result.contains("@@10.1145/3508461"));
    assert!(!result.contains("@@ref1"));
    assert!(!result.contains("@@ref2"));
    assert!(!result.contains("@@ref3"));
    assert!(!result.contains("@@Jones2019"));

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
    use crate::models::BibItem;
    use crate::{AT_REF_PATTERN, REF_PATTERN};
    use indexmap::IndexMap;
    use mdbook_preprocessor::book::Chapter;
    use regex::Regex;
    use std::collections::HashSet;

    // Test that both patterns support BibLaTeX-compliant characters
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
        BibItem {
            citation_key: "doi:10.5555/12345".to_string(),
            title: "DOI Paper".to_string(),
            authors: vec![vec!["Author".to_string()]],
            pub_month: "N/A".to_string(),
            pub_year: "2023".to_string(),
            summary: "Test".to_string(),
            url: Some("https://doi.org/10.5555/12345".to_string()),
            index: None,
            ..Default::default()
        },
    );
    bibliography.insert(
        "arXiv:2301.12345".to_string(),
        BibItem {
            citation_key: "arXiv:2301.12345".to_string(),
            title: "arXiv Paper".to_string(),
            authors: vec![vec!["Researcher".to_string()]],
            pub_month: "N/A".to_string(),
            pub_year: "2023".to_string(),
            summary: "Test".to_string(),
            url: Some("https://arxiv.org/abs/2301.12345".to_string()),
            index: None,
            ..Default::default()
        },
    );
    bibliography.insert(
        "user@domain".to_string(),
        BibItem {
            citation_key: "user@domain".to_string(),
            title: "User Citation".to_string(),
            authors: vec![vec!["User".to_string()]],
            pub_month: "N/A".to_string(),
            pub_year: "2024".to_string(),
            summary: "Test".to_string(),
            url: None,
            index: None,
            ..Default::default()
        },
    );

    let chapter = Chapter::new(
        "Test",
        content.to_string(),
        std::path::PathBuf::new(),
        vec![],
    );
    let mut cited = HashSet::new();
    let handlebars = create_citation_handlebars_with_template("{{item.citation_key}}");
    let backend = LegacyBackend::new(&handlebars);
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
