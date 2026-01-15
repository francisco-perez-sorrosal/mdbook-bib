use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Bibliography item representation.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BibItem {
    // === Core fields (existing, maintained for backward compatibility) ===
    /// The citation key.
    pub citation_key: String,
    /// The article's title.
    pub title: String,
    /// The article's author/s in the format [[Last, First], [Last, First], ...].
    pub authors: Vec<Vec<String>>,
    /// Publication month (1-12 as zero-padded string).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pub_month: Option<String>,
    /// Publication year as string.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pub_year: Option<String>,
    /// Summary/Abstract.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    /// The article's URL.
    pub url: Option<String>,
    /// The item's index for first citation in the book.
    pub index: Option<u32>,

    // === Extended fields from hayagriva (new, all optional for backward compatibility) ===
    /// Entry type (Article, Book, Inproceedings, etc.).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub entry_type: Option<String>,

    /// Digital Object Identifier.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub doi: Option<String>,

    /// Page range (e.g., "123-145").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pages: Option<String>,

    /// Volume number.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub volume: Option<String>,

    /// Issue/Number.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub issue: Option<String>,

    /// Publisher name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub publisher: Option<String>,

    /// Publisher's address/location.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub address: Option<String>,

    /// International Standard Book Number.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub isbn: Option<String>,

    /// International Standard Serial Number.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub issn: Option<String>,

    /// Editor(s) in the same format as authors.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub editor: Option<Vec<Vec<String>>>,

    /// Edition (e.g., "2nd", "Revised").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub edition: Option<String>,

    /// Additional notes.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,

    /// Organization (for conference proceedings).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub organization: Option<String>,

    // === Internal fields for CSL rendering ===
    /// Original hayagriva Entry for CSL rendering.
    /// This is kept for CSL backend to use with BibliographyDriver.
    #[serde(skip)]
    pub hayagriva_entry: Option<Arc<hayagriva::Entry>>,
}

impl BibItem {
    /// Create a new bib item with core fields (for backward compatibility).
    /// Extended fields are initialized to None.
    #[allow(dead_code)]
    pub fn new(
        citation_key: &str,
        title: String,
        authors: Vec<Vec<String>>,
        pub_month: Option<String>,
        pub_year: Option<String>,
        summary: Option<String>,
        url: Option<String>,
    ) -> BibItem {
        BibItem {
            citation_key: citation_key.to_string(),
            title,
            authors,
            pub_month,
            pub_year,
            summary,
            url,
            index: None,
            // Extended fields default to None
            entry_type: None,
            doi: None,
            pages: None,
            volume: None,
            issue: None,
            publisher: None,
            address: None,
            isbn: None,
            issn: None,
            editor: None,
            edition: None,
            note: None,
            organization: None,
            hayagriva_entry: None,
        }
    }
}

/// Citation context for rendering.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Citation {
    pub item: BibItem,
    pub path: String,
}
