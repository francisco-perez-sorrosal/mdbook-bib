use serde::{Deserialize, Serialize};

/// Bibliography item representation.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct BibItem {
    /// The citation key.
    pub citation_key: String,
    /// The article's title.
    pub title: String,
    /// The article's author/s.
    pub authors: Vec<Vec<String>>,
    /// Pub month.
    pub pub_month: String,
    /// Pub year.
    pub pub_year: String,
    /// Summary/Abstract.
    pub summary: String,
    /// The article's url.
    pub url: Option<String>,
    /// The item's index for first citation in the book.
    pub index: Option<u32>,
}

impl BibItem {
    /// Create a new bib item with the provided content.
    #[allow(dead_code)]
    pub fn new(
        citation_key: &str,
        title: String,
        authors: Vec<Vec<String>>,
        pub_month: String,
        pub_year: String,
        summary: String,
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
        }
    }
}

/// Citation context for rendering.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Citation {
    pub item: BibItem,
    pub path: String,
}
