use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::Write;

use tempfile::Builder as TempFileBuilder;

use crate::PlaceholderType::Cite;
use crate::{
    build_bibliography, find_placeholders, load_bibliography, replace_all_placeholders, BibItem,
    Bibiography, Config,
};
use toml::value::Table;
use toml::Value;

use std::convert::TryFrom;

const DUMMY_BIB_SRC: &str = r#"
@misc {fps,
    title = {"This is a bib entry!"},
    author = {"Francisco Perez-Sorrosal"},
    month = {"oct"},
    year = {"2020"},
    what_is_this = {"blabla"},
}
@book{rust_book,
    author = {"Klabnik, Steve and Nichols, Carol"},
    title = {"The Rust Programming Language"},
    year = {"2018"},
    isbn = {"1593278284"},
    publisher = {"No Starch Press"},
}
"#;

const DUMMY_TEXT_WITH_2_VALID_CITE_PLACEHOLDERS: &str = r#"
this is a dumb text that includes citations like {{ #cite fps }} and {{ #cite rust_book }}
"#;

const DUMMY_TEXT_WITH_A_VALID_AND_AN_INVALID_CITE_PLACEHOLDERS: &str = r#"
this is a dumb text that includes valid and invalid citations like {{ #cite fps }} and {{ #cite im_not_there }}
"#;

const DUMMY_TEXT_WITH_2_UNKNOWN_PLACEHOLDERS: &str = r#"
this is a dumb text that includes invalid placeholders like {{ #zoto uhmmmm }} and {{ #peto ahhhhmmm }}
"#;

#[test]
fn load_bib_bibliography_from_file() {
    let temp = TempFileBuilder::new().prefix("book").tempdir().unwrap();
    let chapter_path = temp.path().join("biblio.bib");
    File::create(&chapter_path)
        .unwrap()
        .write_all(DUMMY_BIB_SRC.as_bytes())
        .unwrap();

    let bibliography_loaded: String = load_bibliography(chapter_path.as_path()).unwrap();
    assert_ne!(bibliography_loaded, "");
    assert!(bibliography_loaded.contains("\"Francisco Perez-Sorrosal\""));
}

#[test]
fn cant_load_bib_bibliography_from_file() {
    let temp = TempFileBuilder::new().prefix("book").tempdir().unwrap();
    let chapter_path = temp.path().join("biblio.wrong_extension");
    File::create(&chapter_path)
        .unwrap()
        .write_all(DUMMY_BIB_SRC.as_bytes())
        .unwrap();

    let bibliography_loaded: String = load_bibliography(chapter_path.as_path()).unwrap();
    assert_eq!(bibliography_loaded, "");
}

#[test]
fn bibliography_builder_returns_a_bibliography() {
    let bibliography_loaded: HashMap<String, BibItem> =
        build_bibliography(DUMMY_BIB_SRC.to_string()).unwrap();
    assert_eq!(bibliography_loaded.len(), 2);
    assert_eq!(bibliography_loaded.get("fps").unwrap().citation_key, "fps");
}

#[test]
fn bibliography_render_all_vs_cited() {
    let bibliography_loaded: HashMap<String, BibItem> =
        build_bibliography(DUMMY_BIB_SRC.to_string()).unwrap();

    let mut cited = HashSet::new();
    cited.insert("fps".to_string());

    let html = Bibiography::generate_bibliography_html(&bibliography_loaded, &cited, false);

    assert!(html.contains("This is a bib entry!"));
    assert!(html.contains("The Rust Programming Language"));

    let html = Bibiography::generate_bibliography_html(&bibliography_loaded, &cited, true);

    assert!(html.contains("This is a bib entry!"));
    assert!(!html.contains("The Rust Programming Language"));
}

#[test]
fn valid_and_invalid_citations_are_replaced_properly_in_book_text() {
    let bibliography: HashMap<String, BibItem> =
        build_bibliography(DUMMY_BIB_SRC.to_string()).unwrap();

    let mut cited: HashSet<String> = HashSet::new();

    // Check valid references included in a dummy text
    let text_with_citations = replace_all_placeholders(
        DUMMY_TEXT_WITH_2_VALID_CITE_PLACEHOLDERS,
        &bibliography,
        &mut cited,
    );
    // TODO: These asserts will probably fail if we allow users to specify the bibliography
    // chapter name as per issue #6
    assert!(text_with_citations.contains("[fps](bibliography.html#fps)"));
    assert!(text_with_citations.contains("[rust_book](bibliography.html#rust_book)"));

    // Check a mix of valid and invalid references included/not included in a dummy text
    let text_with_citations = replace_all_placeholders(
        DUMMY_TEXT_WITH_A_VALID_AND_AN_INVALID_CITE_PLACEHOLDERS,
        &bibliography,
        &mut cited,
    );
    assert!(text_with_citations.contains("[fps]"));
    assert!(text_with_citations.contains("[Unknown bib ref:"));
}

#[test]
fn find_only_citation_placeholders() {
    // As long as placeholders are related to cites, they are found, independently of whether they
    // are valid or not
    let plhs = find_placeholders(DUMMY_TEXT_WITH_A_VALID_AND_AN_INVALID_CITE_PLACEHOLDERS);
    let mut items = 0;
    for plh in plhs {
        match plh.placeholder_type {
            Cite(_) => items += 1,
        };
    }
    assert_eq!(items, 2);

    // When no recognized placeholders are found, they are ignored
    let plhs = find_placeholders(DUMMY_TEXT_WITH_2_UNKNOWN_PLACEHOLDERS);
    items = 0;
    for _ in plhs {
        panic!("Only Cite should be recognized as placeholder type!!!");
    }
    assert_eq!(items, 0);
}

#[test]
fn check_config_attributes() {
    let mut t: Table = Table::new();

    // Check config with default values is returned when an empty config is passed in a toml table!!!
    match Config::try_from(Some(&t)) {
        Ok(config) => {
            println!("{:?}", config);
            assert_eq!(config.bibliography, None);
            assert_eq!(config.zotero_user_id, None);
            assert_eq!(config.cited_only, true);
        }
        Err(_) => panic!("there's supposed to be always a config!!!"),
    }

    // Check config attributes are processed (those that are not specified are ignored)!!!
    t.insert(
        "bibliography".to_string(),
        Value::String("biblio.bib".to_string()),
    );
    t.insert(
        "zotero_user_id".to_string(),
        Value::String("123456".to_string()),
    );
    t.insert("render-bib".to_string(), Value::String("all".to_string()));
    t.insert(
        "not-specified-config-attr".to_string(),
        Value::String("uhg???".to_string()),
    );
    match Config::try_from(Some(&t)) {
        Ok(config) => {
            println!("{:?}", config);
            assert_eq!(config.bibliography, Some("biblio.bib"));
            assert_eq!(config.zotero_user_id, Some("123456"));
            assert_eq!(config.cited_only, false);
        }
        Err(_) => panic!("there's supposed to be always a config!!!"),
    }

    // Intentionally add a failure specifying a non-existing value for render-bib
    t.insert(
        "render-bib".to_string(),
        Value::String("non-existent!".to_string()),
    );
    match Config::try_from(Some(&t)) {
        Ok(_) => panic!("there's supposed to be a failure in the config!!!"),
        Err(_) => println!("Yayyyyy! A failure that is supposed to happen!"),
    }
}
