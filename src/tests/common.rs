//! Shared test utilities, fixtures, and helpers.
//!
//! This module provides common functionality used across all test modules:
//! - Test fixtures (BibTeX sources, sample text)
//! - Builder patterns for test data
//! - Handlebars and backend factory functions
//! - File assertion utilities

use crate::backend::CustomBackend;
use crate::config::{DEFAULT_CITE_HB_TEMPLATE, DEFAULT_HB_TEMPLATE};
use crate::models::BibItem;
use crate::parser::{self, BibFormat};
use handlebars::Handlebars;
use indexmap::IndexMap;
use std::path::PathBuf;
use std::sync::OnceLock;

// =============================================================================
// Test Fixtures - BibTeX Sources
// =============================================================================

pub const DUMMY_BIB_SRC: &str = r#"
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

pub const YAML_BIB_SRC: &str = r#"
smith2024:
  type: article
  title: A YAML Bibliography Entry
  author: Smith, John
  date: 2024-03
  parent:
    type: periodical
    title: Journal of YAML Studies
    volume: 5
    issue: 2

jones2023:
  type: book
  title: "The Complete Guide to Bibliography Systems"
  author:
    - Jones, Alice
    - Brown, Bob
  date: 2023
  publisher: Academic Press
  location: Cambridge
  isbn: 978-1234567890
"#;

// =============================================================================
// Test Fixtures - Sample Text with Citations
// =============================================================================

pub const DUMMY_TEXT_WITH_2_VALID_CITE_PLACEHOLDERS: &str = r#"
this is a dumb text that includes citations like {{ #cite fps }} and {{ #cite rust_book }}
"#;

pub const DUMMY_TEXT_WITH_A_VALID_AND_AN_INVALID_CITE_PLACEHOLDERS: &str = r#"
this is a dumb text that includes valid and invalid citations like {{ #cite fps }} and {{ #cite im_not_there }}
"#;

// =============================================================================
// Test Fixtures - Template Examples
// =============================================================================

pub static EXAMPLE_CSS_TEMPLATE: &str = include_str!("../../manual/src/render/my_style.css");
pub static EXAMPLE_HB_TEMPLATE: &str = include_str!("../../manual/src/render/my_references.hbs");

// =============================================================================
// Pre-parsed Bibliographies (Lazy Initialization)
// =============================================================================

/// Returns a pre-parsed version of DUMMY_BIB_SRC for tests that don't need to test parsing.
pub fn dummy_bibliography() -> IndexMap<String, BibItem> {
    static BIBLIOGRAPHY: OnceLock<IndexMap<String, BibItem>> = OnceLock::new();
    BIBLIOGRAPHY
        .get_or_init(|| {
            parser::parse_bibliography(DUMMY_BIB_SRC.to_string(), BibFormat::BibTeX).unwrap()
        })
        .clone()
}

/// Returns a pre-parsed YAML bibliography for tests.
pub fn yaml_bibliography() -> IndexMap<String, BibItem> {
    static BIBLIOGRAPHY: OnceLock<IndexMap<String, BibItem>> = OnceLock::new();
    BIBLIOGRAPHY
        .get_or_init(|| {
            parser::parse_bibliography(YAML_BIB_SRC.to_string(), BibFormat::Yaml).unwrap()
        })
        .clone()
}

// =============================================================================
// Handlebars Factory Functions
// =============================================================================

/// Creates a Handlebars instance with the default references template.
pub fn create_references_handlebars() -> Handlebars<'static> {
    let mut handlebars = Handlebars::new();
    handlebars
        .register_template_string("references", format!("\n\n{DEFAULT_HB_TEMPLATE}\n\n"))
        .unwrap();
    handlebars
}

/// Creates a Handlebars instance with the default citation template.
pub fn create_citation_handlebars() -> Handlebars<'static> {
    let mut handlebars = Handlebars::new();
    handlebars
        .register_template_string("citation", DEFAULT_CITE_HB_TEMPLATE)
        .unwrap();
    handlebars
}

/// Creates a Handlebars instance with a custom citation template.
pub fn create_citation_handlebars_with_template(template: &str) -> Handlebars<'static> {
    let mut handlebars = Handlebars::new();
    handlebars
        .register_template_string("citation", template)
        .unwrap();
    handlebars
}

// =============================================================================
// Backend Factory Functions
// =============================================================================

/// Creates a CustomBackend with the default references template.
pub fn create_references_backend() -> CustomBackend<'static> {
    // We need to leak the Handlebars to get a 'static lifetime
    // This is acceptable in tests since they're short-lived
    let handlebars = Box::leak(Box::new(create_references_handlebars()));
    CustomBackend::new(handlebars)
}

/// Creates a CustomBackend with the default citation template.
pub fn create_citation_backend() -> CustomBackend<'static> {
    let handlebars = Box::leak(Box::new(create_citation_handlebars()));
    CustomBackend::new(handlebars)
}

/// Creates a CustomBackend with a custom citation template.
pub fn create_citation_backend_with_template(template: &str) -> CustomBackend<'static> {
    let handlebars = Box::leak(Box::new(create_citation_handlebars_with_template(template)));
    CustomBackend::new(handlebars)
}

// =============================================================================
// BibItem Builder
// =============================================================================

/// Builder for creating BibItem instances in tests.
///
/// # Example
/// ```ignore
/// let item = BibItemBuilder::new("test2024")
///     .title("Test Article")
///     .authors(&["Smith, John", "Doe, Jane"])
///     .year("2024")
///     .build();
/// ```
#[derive(Default)]
#[allow(dead_code)]
pub struct BibItemBuilder {
    citation_key: String,
    title: String,
    authors: Vec<Vec<String>>,
    pub_month: Option<String>,
    pub_year: Option<String>,
    summary: Option<String>,
    url: Option<String>,
    index: Option<u32>,
    entry_type: Option<String>,
    doi: Option<String>,
    isbn: Option<String>,
    issn: Option<String>,
    volume: Option<String>,
    issue: Option<String>,
    pages: Option<String>,
    publisher: Option<String>,
    address: Option<String>,
    organization: Option<String>,
    editor: Option<Vec<Vec<String>>>,
    edition: Option<String>,
    note: Option<String>,
}

#[allow(dead_code)]
impl BibItemBuilder {
    pub fn new(citation_key: &str) -> Self {
        Self {
            citation_key: citation_key.to_string(),
            ..Default::default()
        }
    }

    /// Creates a builder pre-configured for an article entry.
    pub fn article(citation_key: &str) -> Self {
        Self::new(citation_key).entry_type("Article")
    }

    /// Creates a builder pre-configured for a book entry.
    pub fn book(citation_key: &str) -> Self {
        Self::new(citation_key).entry_type("Book")
    }

    /// Creates a builder pre-configured for a misc entry.
    pub fn misc(citation_key: &str) -> Self {
        Self::new(citation_key).entry_type("Misc")
    }

    pub fn title(mut self, title: &str) -> Self {
        self.title = title.to_string();
        self
    }

    /// Sets authors from a slice of "Last, First" formatted strings.
    pub fn authors(mut self, authors: &[&str]) -> Self {
        self.authors = authors
            .iter()
            .map(|a| a.split(", ").map(String::from).collect())
            .collect();
        self
    }

    /// Sets authors from pre-split name parts.
    pub fn authors_parts(mut self, authors: Vec<Vec<String>>) -> Self {
        self.authors = authors;
        self
    }

    pub fn year(mut self, year: &str) -> Self {
        self.pub_year = Some(year.to_string());
        self
    }

    pub fn month(mut self, month: &str) -> Self {
        self.pub_month = Some(month.to_string());
        self
    }

    pub fn summary(mut self, summary: &str) -> Self {
        self.summary = Some(summary.to_string());
        self
    }

    pub fn url(mut self, url: &str) -> Self {
        self.url = Some(url.to_string());
        self
    }

    pub fn index(mut self, index: u32) -> Self {
        self.index = Some(index);
        self
    }

    pub fn entry_type(mut self, entry_type: &str) -> Self {
        self.entry_type = Some(entry_type.to_string());
        self
    }

    pub fn doi(mut self, doi: &str) -> Self {
        self.doi = Some(doi.to_string());
        self
    }

    pub fn isbn(mut self, isbn: &str) -> Self {
        self.isbn = Some(isbn.to_string());
        self
    }

    pub fn issn(mut self, issn: &str) -> Self {
        self.issn = Some(issn.to_string());
        self
    }

    pub fn volume(mut self, volume: &str) -> Self {
        self.volume = Some(volume.to_string());
        self
    }

    pub fn issue(mut self, issue: &str) -> Self {
        self.issue = Some(issue.to_string());
        self
    }

    pub fn pages(mut self, pages: &str) -> Self {
        self.pages = Some(pages.to_string());
        self
    }

    pub fn publisher(mut self, publisher: &str) -> Self {
        self.publisher = Some(publisher.to_string());
        self
    }

    pub fn organization(mut self, organization: &str) -> Self {
        self.organization = Some(organization.to_string());
        self
    }

    pub fn editor(mut self, editors: &[&str]) -> Self {
        self.editor = Some(
            editors
                .iter()
                .map(|e| e.split(", ").map(String::from).collect())
                .collect(),
        );
        self
    }

    pub fn build(self) -> BibItem {
        BibItem {
            citation_key: self.citation_key,
            title: self.title,
            authors: self.authors,
            pub_month: self.pub_month,
            pub_year: self.pub_year,
            summary: self.summary,
            url: self.url,
            index: self.index,
            entry_type: self.entry_type,
            doi: self.doi,
            isbn: self.isbn,
            issn: self.issn,
            volume: self.volume,
            issue: self.issue,
            pages: self.pages,
            publisher: self.publisher,
            address: self.address,
            organization: self.organization,
            editor: self.editor,
            edition: self.edition,
            note: self.note,
            hayagriva_entry: None,
        }
    }
}

// =============================================================================
// File Assertion Utilities
// =============================================================================

pub struct NotFound;

/// Check if a string is present in the file contents.
pub fn find_str_in_file(input: &str, file: PathBuf) -> Result<(), NotFound> {
    let text = std::fs::read_to_string(file).unwrap();

    for line in text.lines() {
        if line.contains(input) {
            return Ok(());
        }
    }
    Err(NotFound)
}
