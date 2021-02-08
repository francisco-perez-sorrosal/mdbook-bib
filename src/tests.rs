use std::collections::HashMap;
use std::fs::File;
use std::io::Write;

use tempfile::Builder as TempFileBuilder;

use crate::{load_bibliography, BibItem};

const DUMMY_BIB_SRC: &str = "
@misc {fps,
    title = \"This is a bib entry!\",
    author = \"Francisco Perez-Sorrosal\",
    month = \"oct\",
    year = \"2020\",
    polla = \"blabla\",
}
";

#[test]
fn load_bib_bibliography_from_file() {
    let temp = TempFileBuilder::new().prefix("book").tempdir().unwrap();
    let chapter_path = temp.path().join("biblio.bib");
    File::create(&chapter_path)
        .unwrap()
        .write_all(DUMMY_BIB_SRC.as_bytes())
        .unwrap();

    let bibliography_loaded: HashMap<String, BibItem> =
        load_bibliography(chapter_path.as_path()).unwrap();
    assert_eq!(bibliography_loaded.len(), 1);
    assert_eq!(
        bibliography_loaded.get("fps").unwrap().citation_key,
        "fps".to_owned()
    );
}

#[test]
fn cant_load_bib_bibliography_from_file() {
    let temp = TempFileBuilder::new().prefix("book").tempdir().unwrap();
    let chapter_path = temp.path().join("biblio.wrong_extension");
    File::create(&chapter_path)
        .unwrap()
        .write_all(DUMMY_BIB_SRC.as_bytes())
        .unwrap();

    let bibliography_loaded: HashMap<String, BibItem> =
        load_bibliography(chapter_path.as_path()).unwrap();
    assert_eq!(bibliography_loaded.len(), 0);
}
