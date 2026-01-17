//! CSL backend using hayagriva's BibliographyDriver for standardized formatting.
//!
//! This backend provides full CSL (Citation Style Language) support,
//! allowing users to choose from 80+ bundled citation styles or provide custom CSL files.

use anyhow::anyhow;
use hayagriva::archive::{locales, ArchivedStyle};
use hayagriva::citationberg::{IndependentStyle, Locale, Style};
use hayagriva::{BibliographyDriver, BibliographyRequest, CitationItem, CitationRequest};
use lazy_static::lazy_static;
use mdbook_preprocessor::errors::Result as MdResult;
use regex::Regex;

use crate::models::BibItem;

use super::hayagriva_style::{find_style_info, supported_style_aliases, StyleInfo};
use super::{BibliographyBackend, CitationContext};

lazy_static! {
    static ref ANSI_REGEX: Regex = Regex::new(r"\x1b\[[0-9;]*m").unwrap();
}

/// CSL backend using hayagriva's BibliographyDriver.
///
/// This backend renders citations and bibliographies using Citation Style Language (CSL) styles.
/// It supports both bundled styles from the hayagriva archive and custom CSL files.
pub struct CslBackend {
    #[allow(dead_code)]
    style_name: String,
    style: IndependentStyle,
    locales: Vec<Locale>,
    /// Cached style info for quick access to numeric/superscript flags
    style_info: Option<&'static StyleInfo>,
}

impl CslBackend {
    /// Create a new CslBackend with the specified CSL style.
    ///
    /// # Arguments
    /// * `style_name` - Name of CSL style (e.g., "apa", "ieee", "chicago-author-date")
    ///   or path to a custom .csl file
    ///
    /// # Returns
    /// * `Result<Self>` - The configured backend or an error if the style cannot be loaded
    ///
    /// # Supported Bundled Styles
    /// The backend includes 80+ bundled styles from the hayagriva archive.
    /// Use `supported_style_aliases()` to list common aliases.
    pub fn new(style_name: String) -> anyhow::Result<Self> {
        tracing::info!("Initializing CSL backend with style: {}", style_name);

        // Look up in registry first (provides aliases), then fall back to hayagriva's by_name
        let style_info = find_style_info(&style_name);
        let (style, resolved_info) = Self::load_style(&style_name, style_info)?;

        let locales = locales();

        tracing::info!(
            "CSL backend initialized successfully with style '{}'",
            style_name
        );

        Ok(Self {
            style_name,
            style,
            locales,
            style_info: resolved_info,
        })
    }

    /// Load a CSL style from the bundled archive or from a file path.
    fn load_style(
        style_name: &str,
        style_info: Option<&'static StyleInfo>,
    ) -> anyhow::Result<(IndependentStyle, Option<&'static StyleInfo>)> {
        // If found in registry, use the archived style from there
        let archived_style = if let Some(info) = style_info {
            tracing::debug!("Style '{}' found in registry", style_name);
            Some(info.archived)
        } else {
            // Fall back to hayagriva's by_name for styles not in our alias registry
            ArchivedStyle::by_name(style_name)
        };

        let archived_style = archived_style.ok_or_else(|| {
            let aliases: Vec<_> = supported_style_aliases().collect();
            anyhow!(
                "Style '{style_name}' not found. Custom .csl files not yet supported.\n\
                Supported aliases: {}\n\
                See https://francisco-perez-sorrosal.github.io/mdbook-bib/csl.html for the full list.",  // TODO Fix this link
                aliases.join(", ")
            )
        })?;

        tracing::debug!("Loading bundled CSL style: {:?}", archived_style);

        let style = archived_style.get();
        match style {
            Style::Independent(independent) => Ok((independent, style_info)),
            Style::Dependent(_) => Err(anyhow!(
                "Style '{style_name}' is a dependent style. Please use an independent style instead."
            )),
        }
    }

    /// Check if the CSL style uses numeric citations (vs. author-date)
    fn is_numeric_citation_style(&self) -> bool {
        self.style_info.is_some_and(|info| info.numeric)
    }

    /// Check if the CSL style uses superscript citations (vs. bracketed)
    fn is_superscript_citation_style(&self) -> bool {
        self.style_info.is_some_and(|info| info.superscript)
    }

    /// Strip ANSI escape codes from text.
    /// Hayagriva outputs formatted text with ANSI codes for terminal display,
    /// which need to be removed for HTML output.
    fn strip_ansi_codes(text: &str) -> String {
        let result = ANSI_REGEX.replace_all(text, "");

        // Also remove bare ANSI codes that appear without ESC (hayagriva quirk)
        // Only match specific known ANSI codes to avoid stripping legitimate brackets
        result
            .replace("[0m", "") // reset
            .replace("[1m", "") // bold
            .replace("[2m", "") // dim
            .replace("[3m", "") // italic
            .replace("[4m", "") // underline
            .replace("[22m", "") // normal intensity
            .replace("[23m", "") // not italic
            .replace("[24m", "") // not underline
    }
}

impl BibliographyBackend for CslBackend {
    fn format_citation(&self, item: &BibItem, context: &CitationContext) -> MdResult<String> {
        // Determine citation style characteristics
        let is_numeric_style = self.is_numeric_citation_style();
        let is_superscript_style = self.is_superscript_citation_style();

        let linked_citation = if is_numeric_style {
            // For numbered styles, use the assigned index
            // The index is set by citation/mod.rs based on order of first appearance
            let index = item.index.unwrap_or(1);

            if is_superscript_style {
                // Superscript styles (Nature, Vancouver-superscript, etc.)
                // Render as: <sup><a href="url">1</a></sup>
                format!(
                    "<sup><a href=\"{}#{}\">{}</a></sup>",
                    context.bib_page_path, item.citation_key, index
                )
            } else {
                // Bracketed numeric styles (IEEE, Vancouver, etc.)
                // Render as: [[1](url)] → [<a href="url">1</a>]
                format!(
                    "[[{}]({}#{})]",
                    index, context.bib_page_path, item.citation_key
                )
            }
        } else {
            // For author-date styles (Chicago, APA, Harvard, etc.), use hayagriva formatting
            let entry = item.hayagriva_entry.as_ref().ok_or_else(|| {
                anyhow!(
                    "BibItem '{}' missing hayagriva_entry for CSL rendering",
                    item.citation_key
                )
            })?;

            // Create a bibliography driver for this single citation
            let mut driver = BibliographyDriver::new();
            let citation_item = CitationItem::with_entry(entry.as_ref());
            let citation_request =
                CitationRequest::from_items(vec![citation_item], &self.style, &self.locales);
            driver.citation(citation_request);

            let bib_request = BibliographyRequest::new(&self.style, None, &self.locales);
            let rendered = driver.finish(bib_request);

            // Extract and clean the citation
            let citation_text = rendered
                .citations
                .first()
                .map(|c| c.citation.to_string())
                .unwrap_or_else(|| format!("({})", item.citation_key));

            let clean_citation = Self::strip_ansi_codes(&citation_text);

            // Author-date citations are typically in parentheses
            let citation_content = clean_citation.trim_start_matches('(').trim_end_matches(')');

            format!(
                "([{}]({}#{}))",
                citation_content, context.bib_page_path, item.citation_key
            )
        };

        Ok(linked_citation)
    }

    fn format_reference(&self, item: &BibItem) -> MdResult<String> {
        // Get the hayagriva Entry from the BibItem
        let entry = item.hayagriva_entry.as_ref().ok_or_else(|| {
            anyhow!(
                "BibItem '{}' missing hayagriva_entry for CSL rendering",
                item.citation_key
            )
        })?;

        // Create a bibliography driver
        let mut driver = BibliographyDriver::new();

        // Create a citation request to include this entry in the bibliography
        let citation_item = CitationItem::with_entry(entry.as_ref());
        let citation_request =
            CitationRequest::from_items(vec![citation_item], &self.style, &self.locales);

        // Register the citation request
        driver.citation(citation_request);

        // Finish and get the rendered bibliography
        let bib_request = BibliographyRequest::new(&self.style, None, &self.locales);
        let rendered = driver.finish(bib_request);

        // Extract the bibliography entry for this item
        let bib_html = rendered
            .bibliography
            .map(|bib| {
                bib.items
                    .first()
                    .map(|item| item.content.to_string())
                    .unwrap_or_else(|| format!("{} (no bibliography entry)", item.citation_key))
            })
            .unwrap_or_else(|| format!("{} (no bibliography)", item.citation_key));

        // Strip ANSI codes from hayagriva output
        let clean_html = Self::strip_ansi_codes(&bib_html);

        // For numbered styles, prepend the index number
        let is_numeric = self.is_numeric_citation_style();
        let is_superscript = self.is_superscript_citation_style();
        let formatted_entry = if is_numeric {
            let index = item.index.unwrap_or(1);
            if is_superscript {
                // Nature and similar styles use "1." format
                format!("{index}. {clean_html}")
            } else {
                // IEEE and similar styles use "[1]" format
                format!("[{index}] {clean_html}")
            }
        } else {
            clean_html
        };

        // Wrap in a div with CSL entry class and add anchor for linking
        Ok(format!(
            "<div class='csl-entry' id='{}'>{}</div>",
            item.citation_key, formatted_entry
        ))
    }

    fn name(&self) -> &str {
        "CSL"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[test]
    fn test_csl_backend_creation() {
        let backend = CslBackend::new("ieee".to_string());
        assert!(
            backend.is_ok(),
            "Failed to create IEEE backend: {:?}",
            backend.err()
        );

        let backend = backend.unwrap();
        assert_eq!(backend.name(), "CSL");
        assert_eq!(backend.style_name, "ieee");
    }

    #[test]
    fn test_csl_backend_with_apa_style() {
        let backend = CslBackend::new("apa".to_string());
        assert!(
            backend.is_ok(),
            "Failed to create APA backend: {:?}",
            backend.err()
        );
    }

    #[test]
    fn test_csl_backend_with_chicago() {
        let backend = CslBackend::new("chicago-author-date".to_string());
        assert!(
            backend.is_ok(),
            "Failed to create Chicago backend: {:?}",
            backend.err()
        );
    }

    #[test]
    fn test_csl_backend_with_invalid_style() {
        let backend = CslBackend::new("invalid_style_name".to_string());
        assert!(backend.is_err(), "Should fail with invalid style name");
    }

    #[test]
    fn test_format_citation_ieee() {
        let backend = CslBackend::new("ieee".to_string()).expect("Failed to create backend");

        // Create a BibItem with a hayagriva Entry
        let entry_str = r#"@article{test2024,
            author = {Smith, John},
            title = {A Test Article},
            journal = {Test Journal},
            year = {2024},
        }"#;

        let bibliography = hayagriva::io::from_biblatex_str(entry_str).unwrap();
        let entry = bibliography.iter().next().unwrap();

        let item = BibItem {
            citation_key: "test2024".to_string(),
            title: "A Test Article".to_string(),
            hayagriva_entry: Some(Arc::new(entry.clone())),
            ..Default::default()
        };

        let context = CitationContext {
            bib_page_path: "bibliography.html".to_string(),
            chapter_path: "chapter1.md".to_string(),
        };

        let citation = backend.format_citation(&item, &context);
        assert!(
            citation.is_ok(),
            "Citation formatting failed: {:?}",
            citation.err()
        );

        let citation_text = citation.unwrap();
        tracing::info!("IEEE citation: {}", citation_text);
        // IEEE typically uses [1] style citations
        assert!(!citation_text.is_empty(), "Citation should not be empty");
    }

    #[test]
    fn test_format_reference_apa() {
        let backend = CslBackend::new("apa".to_string()).expect("Failed to create backend");

        // Create a BibItem with a hayagriva Entry
        let entry_str = r#"@article{smith2024,
            author = {Smith, John and Doe, Jane},
            title = {Research on Bibliography Systems},
            journal = {Journal of Documentation},
            year = {2024},
            volume = {10},
            pages = {123-145},
        }"#;

        let bibliography = hayagriva::io::from_biblatex_str(entry_str).unwrap();
        let entry = bibliography.iter().next().unwrap();

        let item = BibItem {
            citation_key: "smith2024".to_string(),
            title: "Research on Bibliography Systems".to_string(),
            hayagriva_entry: Some(Arc::new(entry.clone())),
            ..Default::default()
        };

        let reference = backend.format_reference(&item);
        assert!(
            reference.is_ok(),
            "Reference formatting failed: {:?}",
            reference.err()
        );

        let ref_text = reference.unwrap();
        tracing::info!("APA reference: {}", ref_text);
        assert!(
            ref_text.contains("class='csl-entry'"),
            "Should have CSL entry class"
        );
        assert!(!ref_text.is_empty(), "Reference should not be empty");
    }

    #[test]
    fn test_format_citation_nature() {
        let backend = CslBackend::new("nature".to_string()).expect("Failed to create backend");

        let entry_str = r#"@article{watson1953,
            author = {Watson, James D. and Crick, Francis H.C.},
            title = {Molecular Structure of Nucleic Acids: A Structure for Deoxyribose Nucleic Acid},
            journal = {Nature},
            year = {1953},
            volume = {171},
            pages = {737-738},
        }"#;

        let bibliography = hayagriva::io::from_biblatex_str(entry_str).unwrap();
        let entry = bibliography.iter().next().unwrap();

        let item = BibItem {
            citation_key: "watson1953".to_string(),
            title: "Molecular Structure of Nucleic Acids".to_string(),
            hayagriva_entry: Some(Arc::new(entry.clone())),
            ..Default::default()
        };

        let context = CitationContext {
            bib_page_path: "bibliography.html".to_string(),
            chapter_path: "chapter1.md".to_string(),
        };

        let citation = backend.format_citation(&item, &context);
        assert!(citation.is_ok(), "Citation formatting failed");

        let citation_text = citation.unwrap();
        tracing::info!("Nature citation: {}", citation_text);
        assert!(!citation_text.is_empty(), "Citation should not be empty");
    }

    #[test]
    fn test_ansi_stripping_debug() {
        let backend = CslBackend::new("ieee".to_string()).unwrap();

        let entry_str = r#"@article{test2024,
            author = {Smith, John},
            title = {Test},
            journal = {Test Journal},
            year = {2024},
        }"#;

        let bibliography = hayagriva::io::from_biblatex_str(entry_str).unwrap();
        let entry = bibliography.iter().next().unwrap();

        // Create a citation request directly to see raw output
        let mut driver = BibliographyDriver::new();
        let citation_item = CitationItem::with_entry(entry);
        let citation_request =
            CitationRequest::from_items(vec![citation_item], &backend.style, &backend.locales);
        driver.citation(citation_request);

        let bib_request = BibliographyRequest::new(&backend.style, None, &backend.locales);
        let rendered = driver.finish(bib_request);

        if let Some(citation) = rendered.citations.first() {
            let raw = citation.citation.to_string();
            println!("\n=== RAW OUTPUT ===");
            println!("String: {raw:?}");
            println!("Bytes: {:?}", raw.as_bytes());
            println!("Contains [0m: {}", raw.contains("[0m"));

            let stripped = CslBackend::strip_ansi_codes(&raw);
            println!("\n=== AFTER STRIPPING ===");
            println!("String: {stripped:?}");
            println!("Bytes: {:?}", stripped.as_bytes());

            // Verify stripping worked
            assert!(
                !stripped.contains("[0m"),
                "Stripped output should not contain [0m"
            );
            assert!(
                !stripped.contains("[3m"),
                "Stripped output should not contain [3m"
            );
            assert_eq!(stripped, "[1]", "IEEE citation should be [1]");
        }
    }

    #[test]
    fn test_format_citation_output_clean() {
        // End-to-end test: verify format_citation returns clean output
        let backend = CslBackend::new("ieee".to_string()).unwrap();

        let entry_str = r#"@article{test2024,
            author = {Smith, John},
            title = {Test},
            journal = {Test Journal},
            year = {2024},
        }"#;

        let bibliography = hayagriva::io::from_biblatex_str(entry_str).unwrap();
        let entry = bibliography.iter().next().unwrap();

        let item = BibItem {
            citation_key: "test2024".to_string(),
            title: "Test".to_string(),
            hayagriva_entry: Some(Arc::new(entry.clone())),
            ..Default::default()
        };

        let context = CitationContext {
            bib_page_path: "bibliography.html".to_string(),
            chapter_path: "chapter1.md".to_string(),
        };

        let result = backend.format_citation(&item, &context).unwrap();
        println!("format_citation result: {result:?}");

        // Verify no ANSI codes in output
        assert!(!result.contains("[0m"), "Output should not contain [0m");
        assert!(!result.contains("[3m"), "Output should not contain [3m");
        assert!(
            !result.contains("\x1b"),
            "Output should not contain ESC character"
        );
    }
}
