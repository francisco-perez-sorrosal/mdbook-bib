//! Bibliography backend abstraction layer.
//!
//! This module provides a trait-based abstraction for different bibliography
//! rendering backends, allowing users to choose between custom Handlebars
//! templates (LegacyBackend) or CSL citation styles (CslBackend).

mod csl;
mod legacy;

pub use csl::CslBackend;
pub use legacy::LegacyBackend;

use crate::models::BibItem;
use mdbook_preprocessor::errors::Result as MdResult;

/// Backend mode determines which rendering system to use.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackendMode {
    /// Legacy mode: Uses custom Handlebars templates for maximum flexibility.
    Legacy,
    /// CSL mode: Uses Citation Style Language for standardized formatting.
    Csl,
}

/// Citation context provides information needed to render inline citations.
#[derive(Debug, Clone)]
pub struct CitationContext {
    /// Relative path from current chapter to bibliography page.
    pub bib_page_path: String,
    /// Path to current chapter (for logging/debugging).
    pub chapter_path: String,
}

/// Trait for bibliography rendering backends.
///
/// Implementations provide different rendering strategies:
/// - LegacyBackend: Uses Handlebars templates for custom formatting
/// - CslBackend: Uses hayagriva's CSL driver for standardized formatting
pub trait BibliographyBackend {
    /// Format an inline citation reference.
    ///
    /// This generates the HTML that appears in the text when citing a source,
    /// e.g., `[Smith2020]` or `(Smith, 2020)` depending on the backend.
    ///
    /// # Arguments
    /// * `item` - The bibliography item being cited
    /// * `context` - Context information for rendering (paths, etc.)
    fn format_citation(&self, item: &BibItem, context: &CitationContext) -> MdResult<String>;

    /// Format a full bibliography entry for the references section.
    ///
    /// This generates the complete HTML for a single entry in the bibliography,
    /// including all metadata, links, and formatting.
    ///
    /// # Arguments
    /// * `item` - The bibliography item to format
    fn format_reference(&self, item: &BibItem) -> MdResult<String>;

    /// Get the backend name for logging and debugging.
    fn name(&self) -> &str;
}
