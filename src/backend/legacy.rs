//! Legacy backend using Handlebars templates for maximum flexibility.
//!
//! This backend preserves the original mdbook-bib behavior, allowing users
//! to customize citation and reference formatting through Handlebars templates.

use handlebars::Handlebars;
use mdbook_preprocessor::errors::Result as MdResult;

use crate::models::{BibItem, Citation};

use super::{BibliographyBackend, CitationContext};

/// Legacy backend using custom Handlebars templates.
///
/// This backend maintains backward compatibility with existing mdbook-bib
/// installations by using the same Handlebars-based rendering system.
///
/// Templates used:
/// - `citation`: For inline citation rendering (e.g., `[Smith2020]`)
/// - `references`: For full bibliography entry rendering
pub struct LegacyBackend<'a> {
    handlebars: &'a Handlebars<'a>,
}

impl<'a> LegacyBackend<'a> {
    /// Create a new LegacyBackend with the provided Handlebars instance.
    ///
    /// The Handlebars instance must have the `citation` and `references`
    /// templates registered before use.
    pub fn new(handlebars: &'a Handlebars<'a>) -> Self {
        Self { handlebars }
    }
}

impl<'a> BibliographyBackend for LegacyBackend<'a> {
    fn format_citation(&self, item: &BibItem, context: &CitationContext) -> MdResult<String> {
        let citation = Citation {
            item: item.clone(),
            path: context.bib_page_path.clone(),
        };

        self.handlebars.render("citation", &citation).map_err(|e| {
            tracing::error!(
                "Failed to render citation for '{}' in {}: {}",
                item.citation_key,
                context.chapter_path,
                e
            );
            e.into()
        })
    }

    fn format_reference(&self, item: &BibItem) -> MdResult<String> {
        self.handlebars.render("references", item).map_err(|e| {
            tracing::error!(
                "Failed to render reference for '{}': {}",
                item.citation_key,
                e
            );
            e.into()
        })
    }

    fn name(&self) -> &str {
        "Legacy (Handlebars)"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use handlebars::Handlebars;

    #[test]
    fn test_legacy_backend_name() {
        let handlebars = Handlebars::new();
        let backend = LegacyBackend::new(&handlebars);
        assert_eq!(backend.name(), "Legacy (Handlebars)");
    }

    #[test]
    fn test_legacy_backend_format_citation() {
        let mut handlebars = Handlebars::new();
        handlebars
            .register_template_string("citation", "[{{item.citation_key}}]")
            .unwrap();

        let backend = LegacyBackend::new(&handlebars);
        let item = BibItem {
            citation_key: "test_key".to_string(),
            title: "Test Title".to_string(),
            ..Default::default()
        };
        let context = CitationContext {
            bib_page_path: "bibliography.html".to_string(),
            chapter_path: "chapter1.md".to_string(),
        };

        let result = backend.format_citation(&item, &context);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "[test_key]");
    }

    #[test]
    fn test_legacy_backend_format_reference() {
        let mut handlebars = Handlebars::new();
        handlebars
            .register_template_string("references", "<div>{{citation_key}}: {{title}}</div>")
            .unwrap();

        let backend = LegacyBackend::new(&handlebars);
        let item = BibItem {
            citation_key: "test_key".to_string(),
            title: "Test Title".to_string(),
            ..Default::default()
        };

        let result = backend.format_reference(&item);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "<div>test_key: Test Title</div>");
    }
}
