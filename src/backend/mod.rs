//! Bibliography backend abstraction layer.
//!
//! This module provides a trait-based abstraction for different bibliography
//! rendering backends, allowing users to choose between custom Handlebars
//! templates (CustomBackend) or CSL citation styles (CslBackend).

mod csl;
mod custom;
mod hayagriva_style;

pub use csl::CslBackend;
pub use custom::CustomBackend;

use crate::models::BibItem;
use mdbook_preprocessor::errors::Result as MdResult;

/// Backend mode determines which rendering system to use.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackendMode {
    /// Custom mode: Uses Handlebars templates for maximum flexibility.
    Custom,
    /// CSL mode: Uses Citation Style Language for standardized formatting.
    Csl,
}

/// Citation variant determines how a citation should be rendered.
///
/// Different citation syntaxes express different intents:
/// - Standard: `{{#cite key}}` or `@@key` - default rendering
/// - AuthorInText: `@key` - author name in text, year in parens (Pandoc)
/// - Parenthetical: `[@key]` - both author and year in parens (Pandoc)
/// - SuppressAuthor: `[-@key]` - only year, author suppressed (Pandoc)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CitationVariant {
    /// Standard citation: `{{#cite key}}` or `@@key`
    #[default]
    Standard,
    /// Author-in-text: `@key` renders as "Smith (2024)"
    AuthorInText,
    /// Parenthetical: `[@key]` renders as "(Smith, 2024)"
    Parenthetical,
    /// Suppress author: `[-@key]` renders as "(2024)"
    SuppressAuthor,
}

impl CitationVariant {
    /// Convert to a string suitable for use in Handlebars templates.
    pub fn as_template_str(&self) -> &'static str {
        match self {
            CitationVariant::Standard => "standard",
            CitationVariant::AuthorInText => "author_in_text",
            CitationVariant::Parenthetical => "parenthetical",
            CitationVariant::SuppressAuthor => "suppress_author",
        }
    }
}

/// Citation context provides information needed to render inline citations.
#[derive(Debug, Clone)]
pub struct CitationContext {
    /// Relative path from current chapter to bibliography page.
    pub bib_page_path: String,
    /// Path to current chapter (for logging/debugging).
    pub chapter_path: String,
    /// How the citation should be rendered (standard, author-in-text, etc.)
    pub variant: CitationVariant,
}

/// Trait for bibliography rendering backends.
///
/// Implementations provide different rendering strategies:
/// - CustomBackend: Uses Handlebars templates for custom formatting
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
