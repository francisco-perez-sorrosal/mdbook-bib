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

use super::hayagriva_style::{
    detect_style_format, find_style_info, supported_style_aliases, CitationContentType,
    CitationFormat, CitationRendering, CitationStyle, DetectedStyleFormat, StyleInfo,
};
use super::{BibliographyBackend, CitationContext, CitationVariant};

lazy_static! {
    static ref ANSI_REGEX: Regex = Regex::new(r"\x1b\[[0-9;]*m").unwrap();
}

/// CSL backend using hayagriva's BibliographyDriver.
///
/// This backend renders citations and bibliographies using Citation Style Language (CSL) styles.
/// It supports both bundled styles from the hayagriva archive and custom CSL files.
///
/// ## Style Resolution
///
/// 1. **Registry styles**: Looked up by alias (e.g., "ieee", "apa") with full metadata
///    including superscript hints.
/// 2. **Fallback styles**: Any style available via `ArchivedStyle::by_name()`. Citation
///    format (numeric vs author-date) is detected from CSL metadata, but superscript
///    cannot be detected and defaults to `false`.
pub struct CslBackend {
    #[allow(dead_code)]
    style_name: String,
    style: IndependentStyle,
    locales: Vec<Locale>,
    /// Style info from registry (if available) - provides aliases and superscript hints
    style_info: Option<&'static StyleInfo>,
    /// Detected format from CSL metadata (used when style_info is None)
    detected_format: DetectedStyleFormat,
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

        // Detect format from CSL metadata (used as fallback when not in registry)
        let detected_format = detect_style_format(&style);
        if resolved_info.is_none() {
            tracing::info!(
                "Style '{}' not in registry, detected format: {:?}",
                style_name,
                detected_format.citation_format()
            );
        }

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
            detected_format,
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
                Full list: https://github.com/typst/hayagriva (use ArchivedStyle names)",
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

    /// Get the effective citation format (registry or detected).
    ///
    /// Returns the `CitationFormat` from `StyleInfo` if available (for registry styles),
    /// otherwise falls back to `DetectedStyleFormat` (for fallback styles).
    fn citation_format(&self) -> CitationFormat {
        match &self.style_info {
            Some(info) => info.citation_format(),
            None => self.detected_format.citation_format(),
        }
    }

    /// Get the citation text from hayagriva for a given item.
    ///
    /// This is used for label and author-date styles where hayagriva generates the text.
    fn get_hayagriva_citation_text(&self, item: &BibItem, fallback: &str) -> MdResult<String> {
        let entry = item.hayagriva_entry.as_ref().ok_or_else(|| {
            anyhow!(
                "BibItem '{}' missing hayagriva_entry for CSL rendering",
                item.citation_key
            )
        })?;

        let mut driver = BibliographyDriver::new();
        let citation_item = CitationItem::with_entry(entry.as_ref());
        let citation_request =
            CitationRequest::from_items(vec![citation_item], &self.style, &self.locales);
        driver.citation(citation_request);

        let bib_request = BibliographyRequest::new(&self.style, None, &self.locales);
        let rendered = driver.finish(bib_request);

        let citation_text = match rendered.citations.first() {
            Some(c) => c.citation.to_string(),
            None => {
                tracing::warn!(
                    "Hayagriva returned no citation for '{}', using fallback",
                    item.citation_key
                );
                fallback.to_string()
            }
        };

        Ok(Self::strip_ansi_codes(&citation_text))
    }

    /// Strip ANSI escape codes from text.
    ///
    /// Hayagriva outputs formatted text with ANSI codes for terminal display,
    /// which need to be removed for HTML output. This handles both standard
    /// escape sequences (`\x1b[...m`) and bare codes (`[0m`, `[3m`, etc.)
    /// that hayagriva sometimes emits without the ESC prefix.
    fn strip_ansi_codes(text: &str) -> String {
        // Standard ANSI escape sequences
        let result = ANSI_REGEX.replace_all(text, "");

        // Bare ANSI codes without ESC prefix (hayagriva quirk)
        const BARE_CODES: &[&str] = &[
            "[0m", "[1m", "[2m", "[3m", "[4m", // basic formatting
            "[22m", "[23m", "[24m", // reset formatting
        ];

        let mut result = result.into_owned();
        for code in BARE_CODES {
            result = result.replace(code, "");
        }
        result
    }

    /// Format a fallback bibliography entry when hayagriva doesn't provide one.
    ///
    /// Some CSL styles (like alphanumeric) don't define a bibliography section,
    /// so we construct a simple entry from the BibItem metadata.
    fn format_fallback_bibliography(item: &BibItem) -> String {
        let mut parts = Vec::new();

        // Authors (format: "LastName, F.")
        if !item.authors.is_empty() {
            let author_str: String = item
                .authors
                .iter()
                .map(|name_parts| {
                    if name_parts.len() >= 2 {
                        let last = &name_parts[0];
                        let first = &name_parts[1];
                        // Use first initial if available, otherwise full first name
                        let initial_part = first
                            .chars()
                            .next()
                            .map(|c| format!("{c}."))
                            .unwrap_or_else(|| first.clone());
                        format!("{last}, {initial_part}")
                    } else if !name_parts.is_empty() {
                        name_parts[0].clone()
                    } else {
                        "Unknown".to_string()
                    }
                })
                .collect::<Vec<_>>()
                .join(" and ");
            parts.push(author_str);
        }

        // Title (in quotes, with period inside)
        if !item.title.is_empty() {
            parts.push(format!("\"{}.\"", item.title));
        }

        // Year
        if let Some(year) = &item.pub_year {
            parts.push(format!("{year}."));
        }

        if parts.is_empty() {
            item.citation_key.clone()
        } else {
            parts.join(" ")
        }
    }
}

impl BibliographyBackend for CslBackend {
    fn format_citation(&self, item: &BibItem, context: &CitationContext) -> MdResult<String> {
        let format = self.citation_format();
        let link = format!("{}#{}", context.bib_page_path, item.citation_key);
        let variant = context.variant;

        // For numeric and label styles, variant doesn't affect content
        // For author-date styles, we need to handle variants differently
        let linked_citation = match format.content {
            CitationContentType::Numeric => {
                let content = item.index.unwrap_or(1).to_string();
                match format.rendering {
                    CitationRendering::Superscript => {
                        format!("<sup><a href=\"{link}\">{content}</a></sup>")
                    }
                    CitationRendering::Bracketed => {
                        format!("[[{content}]({link})]")
                    }
                }
            }
            CitationContentType::Label => {
                // For label styles (alphanumeric), use hayagriva to generate author-based labels
                let content =
                    self.get_hayagriva_citation_text(item, &format!("[{}]", item.citation_key))?;
                let label = content.trim_matches(&['[', ']'] as &[char]);
                format!("[[{label}]({link})]")
            }
            CitationContentType::AuthorDate => {
                // For author-date styles, handle Pandoc citation variants
                let full_citation =
                    self.get_hayagriva_citation_text(item, &format!("({})", item.citation_key))?;
                let full_text = full_citation.trim_matches(&['(', ')'] as &[char]);

                match variant {
                    CitationVariant::Standard | CitationVariant::Parenthetical => {
                        // Standard and parenthetical: "(Smith, 2024)" or "(Smith 2024)"
                        format!("([{full_text}]({link}))")
                    }
                    CitationVariant::AuthorInText => {
                        // Author-in-text: "Smith (2024)" - author outside parens, year linked
                        // Try to split on ", " first (APA style), then space (other styles)
                        if let Some((author, year)) = full_text.split_once(", ") {
                            format!("{author} ([{year}]({link}))")
                        } else if let Some((author, year)) = full_text.split_once(' ') {
                            format!("{author} ([{year}]({link}))")
                        } else {
                            // Fallback if can't split
                            format!("([{full_text}]({link}))")
                        }
                    }
                    CitationVariant::SuppressAuthor => {
                        // Suppress author: "(2024)" - only year, author suppressed
                        // Try to extract just the year part
                        if let Some((_, year)) = full_text.split_once(", ") {
                            format!("([{year}]({link}))")
                        } else if let Some((_, year)) = full_text.split_once(' ') {
                            format!("([{year}]({link}))")
                        } else {
                            // Fallback: just link the whole thing
                            format!("([{full_text}]({link}))")
                        }
                    }
                }
            }
        };

        Ok(linked_citation)
    }

    fn format_reference(&self, item: &BibItem) -> MdResult<String> {
        let format = self.citation_format();

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
            .and_then(|bib| bib.items.first().map(|i| i.content.to_string()));

        // If no bibliography content from hayagriva, construct a fallback
        let bib_content = match bib_html {
            Some(html) => Self::strip_ansi_codes(&html),
            None => Self::format_fallback_bibliography(item),
        };

        // Format entry based on content type and rendering
        let formatted_entry = match (format.content, format.rendering) {
            (CitationContentType::Numeric, CitationRendering::Superscript) => {
                // Nature and similar styles use "1." format
                let index = item.index.unwrap_or(1);
                format!("{index}. {bib_content}")
            }
            (CitationContentType::Numeric, CitationRendering::Bracketed) => {
                // IEEE and similar styles use "[1]" format
                let index = item.index.unwrap_or(1);
                format!("[{index}] {bib_content}")
            }
            (CitationContentType::Label, _) => {
                // For label styles (alphanumeric), get the label from hayagriva citation
                let citation_text = rendered
                    .citations
                    .first()
                    .map(|c| c.citation.to_string())
                    .unwrap_or_else(|| format!("[{}]", item.citation_key));

                let clean_label = Self::strip_ansi_codes(&citation_text);
                // trim_matches handles potential nested brackets
                let label = clean_label.trim_matches(&['[', ']'] as &[char]);

                format!("[{label}] {bib_content}")
            }
            (CitationContentType::AuthorDate, _) => {
                // Author-date styles: no prefix
                bib_content
            }
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
            variant: CitationVariant::Standard,
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
            variant: CitationVariant::Standard,
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
            variant: CitationVariant::Standard,
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

    #[test]
    fn test_fallback_style_format_detection() {
        // Use a style that's in hayagriva but NOT in our registry
        // "annual-reviews" is a numeric style available via ArchivedStyle::by_name()
        let backend = CslBackend::new("annual-reviews".to_string());

        // The style should load successfully via fallback
        assert!(
            backend.is_ok(),
            "Should load non-registry style via fallback: {:?}",
            backend.err()
        );

        let backend = backend.unwrap();

        // style_info should be None (not in registry)
        assert!(
            backend.style_info.is_none(),
            "Non-registry style should have style_info=None"
        );

        // But numeric should be detected from CSL metadata
        // annual-reviews is a numeric style
        assert_eq!(
            backend.citation_format().content,
            CitationContentType::Numeric,
            "annual-reviews should be detected as numeric from CSL metadata"
        );
    }

    #[test]
    fn test_registry_style_has_style_info() {
        let backend = CslBackend::new("ieee".to_string()).unwrap();

        // Registry style should have style_info
        assert!(
            backend.style_info.is_some(),
            "Registry style should have style_info"
        );

        // And should be numeric
        assert_eq!(
            backend.citation_format().content,
            CitationContentType::Numeric,
            "IEEE should be numeric"
        );
    }

    // --- New style integration tests ---

    #[test]
    fn test_vancouver_superscript_citation() {
        let backend =
            CslBackend::new("vancouver-superscript".to_string()).expect("Failed to create backend");

        // Should be numeric AND superscript
        let format = backend.citation_format();
        assert_eq!(
            format.content,
            CitationContentType::Numeric,
            "vancouver-superscript should be numeric"
        );
        assert_eq!(
            format.rendering,
            CitationRendering::Superscript,
            "vancouver-superscript should use superscript"
        );

        // Test citation rendering
        let entry_str = r#"@article{test2024,
            author = {Smith, John},
            title = {Test Article},
            journal = {Test Journal},
            year = {2024},
        }"#;

        let bibliography = hayagriva::io::from_biblatex_str(entry_str).unwrap();
        let entry = bibliography.iter().next().unwrap();

        let item = BibItem {
            citation_key: "test2024".to_string(),
            title: "Test Article".to_string(),
            index: Some(1),
            hayagriva_entry: Some(Arc::new(entry.clone())),
            ..Default::default()
        };

        let context = CitationContext {
            bib_page_path: "bibliography.html".to_string(),
            chapter_path: "chapter1.md".to_string(),
            variant: CitationVariant::Standard,
        };

        let citation = backend.format_citation(&item, &context).unwrap();
        // Superscript styles render as <sup><a href="...">1</a></sup>
        assert!(citation.contains("<sup>"), "Should contain superscript tag");
        assert!(citation.contains("</sup>"), "Should close superscript tag");
    }

    #[test]
    fn test_alphanumeric_citation() {
        let backend =
            CslBackend::new("alphanumeric".to_string()).expect("Failed to create backend");

        // Alphanumeric uses author-based labels (not sequential numbers)
        let format = backend.citation_format();
        assert_eq!(
            format.content,
            CitationContentType::Label,
            "alphanumeric should be a label style"
        );
        assert_eq!(
            format.rendering,
            CitationRendering::Bracketed,
            "alphanumeric should be bracketed"
        );

        // Test citation rendering with an actual entry
        let entry_str = r#"@article{smith2024,
            author = {Smith, John A.},
            title = {Modern Data Analysis},
            journal = {Data Science Journal},
            year = {2024},
        }"#;

        let bibliography = hayagriva::io::from_biblatex_str(entry_str).unwrap();
        let entry = bibliography.iter().next().unwrap();

        let item = BibItem {
            citation_key: "smith2024".to_string(),
            title: "Modern Data Analysis".to_string(),
            hayagriva_entry: Some(Arc::new(entry.clone())),
            ..Default::default()
        };

        let context = CitationContext {
            bib_page_path: "bibliography.html".to_string(),
            chapter_path: "chapter1.md".to_string(),
            variant: CitationVariant::Standard,
        };

        let citation = backend.format_citation(&item, &context).unwrap();
        println!("Alphanumeric citation: {citation}");

        // Should contain an author-based label, not a sequential number
        // The label format is typically [Smi24] or similar
        assert!(
            citation.contains("bibliography.html#smith2024"),
            "Citation should link to bibliography"
        );
        // Should NOT be a plain number like [1]
        assert!(
            !citation.contains(">1<"),
            "Label style should not use sequential numbers"
        );
    }

    #[test]
    fn test_alphanumeric_reference_rendering() {
        let backend =
            CslBackend::new("alphanumeric".to_string()).expect("Failed to create backend");

        let entry_str = r#"@article{smith2024,
            author = {Smith, John A.},
            title = {Modern Data Analysis},
            journal = {Data Science Journal},
            year = {2024},
            volume = {10},
            pages = {1-20},
        }"#;

        let bibliography = hayagriva::io::from_biblatex_str(entry_str).unwrap();
        let entry = bibliography.iter().next().unwrap();

        let item = BibItem {
            citation_key: "smith2024".to_string(),
            title: "Modern Data Analysis".to_string(),
            authors: vec![vec!["Smith".to_string(), "John A.".to_string()]],
            pub_year: Some("2024".to_string()),
            hayagriva_entry: Some(Arc::new(entry.clone())),
            ..Default::default()
        };

        let reference = backend.format_reference(&item).unwrap();
        println!("Alphanumeric reference: {reference}");

        // Should contain the label
        assert!(
            reference.contains("[Smi24]"),
            "Reference should contain the label [Smi24]"
        );

        // Should contain the citation key anchor
        assert!(
            reference.contains("id='smith2024'"),
            "Reference should have anchor for citation key"
        );

        // Should contain author and title from fallback format
        assert!(
            reference.contains("Smith"),
            "Reference should contain author name"
        );
        assert!(
            reference.contains("Modern Data Analysis"),
            "Reference should contain title"
        );
    }

    #[test]
    fn test_elsevier_vancouver_citation() {
        let backend =
            CslBackend::new("elsevier-vancouver".to_string()).expect("Failed to create backend");

        let format = backend.citation_format();
        assert_eq!(
            format.content,
            CitationContentType::Numeric,
            "elsevier-vancouver should be numeric"
        );
        assert_eq!(
            format.rendering,
            CitationRendering::Bracketed,
            "elsevier-vancouver should be bracketed"
        );
    }

    #[test]
    fn test_springer_basic_author_date_citation() {
        let backend = CslBackend::new("springer-basic-author-date".to_string())
            .expect("Failed to create backend");

        let format = backend.citation_format();
        assert_eq!(
            format.content,
            CitationContentType::AuthorDate,
            "springer-basic-author-date should be author-date"
        );
        assert_eq!(
            format.rendering,
            CitationRendering::Bracketed,
            "springer-basic-author-date should be bracketed"
        );
    }

    #[test]
    fn test_mla8_citation() {
        let backend = CslBackend::new("mla8".to_string()).expect("Failed to create backend");

        let format = backend.citation_format();
        assert_eq!(
            format.content,
            CitationContentType::AuthorDate,
            "mla8 should be author-date style"
        );
        assert_eq!(
            format.rendering,
            CitationRendering::Bracketed,
            "mla8 should be bracketed"
        );
    }

    // --- Registry integrity tests ---

    #[test]
    fn test_no_duplicate_aliases() {
        use std::collections::HashSet;
        let mut seen = HashSet::new();

        for style in super::super::hayagriva_style::all_registry_styles() {
            for alias in style.aliases {
                assert!(
                    seen.insert(*alias),
                    "Duplicate alias found in registry: '{alias}'"
                );
            }
        }
    }

    #[test]
    fn test_registry_style_count() {
        let count = super::super::hayagriva_style::registry_style_count();
        assert!(
            count >= 19,
            "Registry should have at least 19 styles, found {count}"
        );
    }

    #[test]
    fn test_format_style_list() {
        let list = super::super::hayagriva_style::format_style_list();

        // Check that key styles appear in correct categories
        assert!(list.contains("ieee"), "Should list ieee");
        assert!(list.contains("apa"), "Should list apa");
        assert!(list.contains("nature"), "Should list nature");
        assert!(list.contains("alphanumeric"), "Should list alphanumeric");

        // Check structure
        assert!(
            list.contains("Numeric styles:"),
            "Should have numeric section"
        );
        assert!(
            list.contains("Superscript styles:"),
            "Should have superscript section"
        );
        assert!(list.contains("Label styles:"), "Should have label section");
        assert!(
            list.contains("Author-date styles:"),
            "Should have author-date section"
        );
    }
}
