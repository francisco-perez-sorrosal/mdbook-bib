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

/// A locator within a cited work (e.g., page number, chapter, section).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CitationLocator {
    /// CSL locator type (e.g., Page, Chapter, Section).
    pub kind: hayagriva::citationberg::taxonomy::Locator,
    /// Locator value (e.g., `"12"`, `"3-5"`, `"2"`).
    pub value: String,
}

/// A single citation item inside a Pandoc bracketed citation group.
#[derive(Debug, Clone)]
pub struct CitationGroupItem<'a> {
    /// The bibliography item being cited.
    pub item: &'a BibItem,
    /// Optional prefix text before this item (e.g., `"see "`).
    pub prefix: Option<String>,
    /// Optional structured locator (e.g., page 12, chapter 2).
    pub locator: Option<CitationLocator>,
    /// Optional suffix text after the citation/locator (e.g., `", and passim"`).
    pub suffix: Option<String>,
    /// Whether to suppress the author name (`[-@key]`).
    pub suppress_author: bool,
    /// Original locator/suffix text as written (used by fallback backends).
    pub raw_locator: Option<String>,
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

    /// Format a bracketed citation group containing one or more items,
    /// optionally with prefixes, locators, suffixes, or suppressed authors.
    ///
    /// Non-CSL backends use this default implementation, which renders each
    /// item individually via [`BibliographyBackend::format_citation`] and joins
    /// them with `"; "` inside a single pair of parentheses.
    fn format_citation_group(
        &self,
        items: &[CitationGroupItem<'_>],
        context: &CitationContext,
    ) -> MdResult<String> {
        if items.is_empty() {
            return Ok(String::new());
        }

        // Preserve single-item legacy output when no group features are used.
        if items.len() == 1
            && items[0].prefix.is_none()
            && items[0].locator.is_none()
            && items[0].suffix.is_none()
            && items[0].raw_locator.is_none()
        {
            let mut item_ctx = context.clone();
            item_ctx.variant = if items[0].suppress_author {
                CitationVariant::SuppressAuthor
            } else {
                CitationVariant::Parenthetical
            };
            return self.format_citation(items[0].item, &item_ctx);
        }

        let mut parts = Vec::with_capacity(items.len());
        for group_item in items {
            let mut item_ctx = context.clone();
            item_ctx.variant = if group_item.suppress_author {
                CitationVariant::SuppressAuthor
            } else {
                CitationVariant::Parenthetical
            };
            let rendered = self.format_citation(group_item.item, &item_ctx)?;
            let mut part = String::new();
            if let Some(prefix) = &group_item.prefix {
                part.push_str(prefix);
            }
            part.push_str(&rendered);
            if let Some(raw_loc) = &group_item.raw_locator {
                part.push_str(raw_loc);
            } else if let Some(suffix) = &group_item.suffix {
                part.push_str(suffix);
            }
            parts.push(part);
        }

        Ok(format!("({})", parts.join("; ")))
    }

    /// Get the backend name for logging and debugging.
    fn name(&self) -> &str;
}
