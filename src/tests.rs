use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::Write;

use tempfile::Builder as TempFileBuilder;

use crate::{
    build_bibliography, load_bibliography, replace_all_placeholders, BibItem, Bibiography,
};

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

const DUMMY_TEXT_WITH_VALID_REFERENCES: &str = r#"
this is a dumb text that includes citations like {{ #cite fps }} and {{ #cite rust_book }}
"#;

const DUMMY_TEXT_WITH_VALID_AND_INVALID_REFERENCES: &str = r#"
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
    let text_with_citations =
        replace_all_placeholders(DUMMY_TEXT_WITH_VALID_REFERENCES, &bibliography, &mut cited);
    // TODO: These asserts will probably fail if we allow users to specify the bibliography
    // chapter name as per issue #6
    assert!(text_with_citations.contains("[fps](bibliography.html#fps)"));
    assert!(text_with_citations.contains("[rust_book](bibliography.html#rust_book)"));

    // Check a mix of valid and invalid references included/not included in a dummy text
    let text_with_citations = replace_all_placeholders(
        DUMMY_TEXT_WITH_VALID_AND_INVALID_REFERENCES,
        &bibliography,
        &mut cited,
    );
    assert!(text_with_citations.contains("[fps]"));
    assert!(text_with_citations.contains("[Unknown bib ref:"));
}
