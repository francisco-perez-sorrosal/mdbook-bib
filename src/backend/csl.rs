//! CSL backend using hayagriva's CitationEngine for standardized formatting.
//!
//! This backend will provide CSL (Citation Style Language) support in Phase 5,
//! allowing users to choose from hundreds of standard citation styles.
//!
//! **Note**: This is a stub implementation for Phase 4. Full implementation
//! comes in Phase 5: CSL Integration.

use mdbook_preprocessor::errors::Result as MdResult;

use crate::models::BibItem;

use super::{BibliographyBackend, CitationContext};

/// CSL backend using hayagriva's CitationEngine.
///
/// **Phase 5 TODO**: Implement actual CSL rendering using hayagriva's
/// BibliographyDriver and CitationEngine.
pub struct CslBackend {
    _style_name: String,
}

impl CslBackend {
    /// Create a new CslBackend with the specified CSL style.
    ///
    /// # Arguments
    /// * `style_name` - Name of CSL style (e.g., "apa", "ieee", "chicago")
    ///
    /// **Phase 5 TODO**: Load actual CSL style file and initialize CitationEngine.
    #[allow(dead_code)]
    pub fn new(style_name: String) -> Self {
        tracing::warn!("CslBackend is a stub in Phase 4. Full CSL support comes in Phase 5.");
        Self {
            _style_name: style_name,
        }
    }
}

impl BibliographyBackend for CslBackend {
    fn format_citation(&self, item: &BibItem, _context: &CitationContext) -> MdResult<String> {
        // Phase 4 stub: Return simple placeholder
        // Phase 5 TODO: Use hayagriva's CitationEngine to format citation
        Ok(format!("[{}]", item.citation_key))
    }

    fn format_reference(&self, item: &BibItem) -> MdResult<String> {
        // Phase 4 stub: Return simple placeholder
        // Phase 5 TODO: Use hayagriva's BibliographyDriver to format full reference
        Ok(format!(
            "<div class='csl-entry'>[{}] - {} (CSL formatting coming in Phase 5)</div>",
            item.citation_key, item.title
        ))
    }

    fn name(&self) -> &str {
        "CSL (Stub)"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_csl_backend_stub() {
        let backend = CslBackend::new("apa".to_string());
        assert_eq!(backend.name(), "CSL (Stub)");

        let item = BibItem {
            citation_key: "test_key".to_string(),
            title: "Test Title".to_string(),
            ..Default::default()
        };
        let context = CitationContext {
            bib_page_path: "bibliography.html".to_string(),
            chapter_path: "chapter1.md".to_string(),
        };

        let citation = backend.format_citation(&item, &context);
        assert!(citation.is_ok());
        assert_eq!(citation.unwrap(), "[test_key]");

        let reference = backend.format_reference(&item);
        assert!(reference.is_ok());
        let ref_text = reference.unwrap();
        assert!(ref_text.contains("test_key"));
        assert!(ref_text.contains("Phase 5"));
    }
}
