//! Hayagriva CSL style registry and metadata.
//!
//! This module provides:
//! - A registry of common citation styles with short aliases (e.g., "ieee", "apa")
//! - Runtime detection of citation format from any CSL style's metadata
//!
//! ## Style Resolution Strategy
//!
//! 1. **Registry lookup**: Check if the style name matches a known alias
//! 2. **Hayagriva fallback**: Try `ArchivedStyle::by_name()` for full style names
//! 3. **Format detection**: Extract numeric/author-date from the style's metadata
//!
//! The registry provides short aliases and superscript hints (not detectable from CSL).
//! For styles not in the registry, we detect numeric vs author-date from CSL metadata.

use hayagriva::archive::ArchivedStyle;
use hayagriva::citationberg::{CitationFormat, IndependentStyle, StyleCategory};

/// Common interface for citation format characteristics.
///
/// Both `StyleInfo` (from registry) and `DetectedStyleFormat` (from CSL metadata)
/// implement this trait to provide uniform access to citation format flags.
pub trait CitationStyle {
    /// Whether the style uses sequential numeric citations (e.g., `[1]`, `[2]`)
    ///
    /// Returns true for IEEE, Vancouver, etc. where we manage the citation index.
    /// Returns false for author-date and label styles where hayagriva generates the text.
    fn is_numeric(&self) -> bool;

    /// Whether the style uses author-based labels (e.g., `[Smi24]`, `[JL23]`)
    ///
    /// Label styles generate unique identifiers from author names and publication year.
    /// Unlike numeric styles, the citation text comes from hayagriva, not from our index.
    fn is_label(&self) -> bool;

    /// Whether the style uses superscript citations (e.g., `¹`, `²`)
    fn is_superscript(&self) -> bool;
}

/// Style metadata from the registry: aliases, archived style, and format hints.
///
/// This is the authoritative source for aliased styles, providing both
/// short names and superscript hints that cannot be detected from CSL metadata.
#[derive(Debug, Clone, Copy)]
pub struct StyleInfo {
    /// Short alias(es) for the style (first is canonical)
    pub aliases: &'static [&'static str],
    /// The hayagriva ArchivedStyle variant
    pub archived: ArchivedStyle,
    /// Whether this style uses sequential numeric citations (e.g., [1], [2])
    numeric: bool,
    /// Whether this style uses author-based labels (e.g., [Smi24])
    label: bool,
    /// Whether this style uses superscript citations (e.g., Nature)
    superscript: bool,
}

impl CitationStyle for StyleInfo {
    fn is_numeric(&self) -> bool {
        self.numeric
    }
    fn is_label(&self) -> bool {
        self.label
    }
    fn is_superscript(&self) -> bool {
        self.superscript
    }
}

/// Registry of supported style aliases with their metadata.
/// This is the single source of truth for style resolution and classification.
static STYLE_REGISTRY: &[StyleInfo] = &[
    StyleInfo {
        aliases: &["ieee"],
        archived: ArchivedStyle::InstituteOfElectricalAndElectronicsEngineers,
        numeric: true,
        label: false,
        superscript: false,
    },
    StyleInfo {
        aliases: &["apa", "american-psychological-association"],
        archived: ArchivedStyle::AmericanPsychologicalAssociation,
        numeric: false,
        label: false,
        superscript: false,
    },
    StyleInfo {
        aliases: &["chicago-author-date"],
        archived: ArchivedStyle::ChicagoAuthorDate,
        numeric: false,
        label: false,
        superscript: false,
    },
    StyleInfo {
        aliases: &["chicago-notes"],
        archived: ArchivedStyle::ChicagoNotes,
        numeric: false,
        label: false,
        superscript: false,
    },
    StyleInfo {
        aliases: &["mla", "modern-language-association"],
        archived: ArchivedStyle::ModernLanguageAssociation,
        numeric: false,
        label: false,
        superscript: false,
    },
    StyleInfo {
        aliases: &["mla8", "modern-language-association-8"],
        archived: ArchivedStyle::ModernLanguageAssociation8,
        numeric: false,
        label: false,
        superscript: false,
    },
    StyleInfo {
        aliases: &["nature"],
        archived: ArchivedStyle::Nature,
        numeric: true,
        label: false,
        superscript: true,
    },
    StyleInfo {
        aliases: &["vancouver"],
        archived: ArchivedStyle::Vancouver,
        numeric: true,
        label: false,
        superscript: false,
    },
    StyleInfo {
        aliases: &["vancouver-superscript"],
        archived: ArchivedStyle::VancouverSuperscript,
        numeric: true,
        label: false,
        superscript: true,
    },
    StyleInfo {
        aliases: &["harvard", "harvard-cite-them-right"],
        archived: ArchivedStyle::HarvardCiteThemRight,
        numeric: false,
        label: false,
        superscript: false,
    },
    StyleInfo {
        aliases: &["acm", "association-for-computing-machinery"],
        archived: ArchivedStyle::AssociationForComputingMachinery,
        numeric: true,
        label: false,
        superscript: false,
    },
    StyleInfo {
        aliases: &["acs", "american-chemical-society"],
        archived: ArchivedStyle::AmericanChemicalSociety,
        numeric: true,
        label: false,
        superscript: false,
    },
    StyleInfo {
        aliases: &["ama", "american-medical-association"],
        archived: ArchivedStyle::AmericanMedicalAssociation,
        numeric: true,
        label: false,
        superscript: false,
    },
    StyleInfo {
        aliases: &["springer-basic"],
        archived: ArchivedStyle::SpringerBasic,
        numeric: true,
        label: false,
        superscript: false,
    },
    StyleInfo {
        aliases: &["springer-basic-author-date"],
        archived: ArchivedStyle::SpringerBasicAuthorDate,
        numeric: false,
        label: false,
        superscript: false,
    },
    StyleInfo {
        aliases: &["cell"],
        archived: ArchivedStyle::Cell,
        numeric: true,
        label: false,
        superscript: false,
    },
    StyleInfo {
        aliases: &["elsevier-harvard"],
        archived: ArchivedStyle::ElsevierHarvard,
        numeric: false,
        label: false,
        superscript: false,
    },
    StyleInfo {
        aliases: &["elsevier-vancouver"],
        archived: ArchivedStyle::ElsevierVancouver,
        numeric: true,
        label: false,
        superscript: false,
    },
    StyleInfo {
        aliases: &["alphanumeric"],
        archived: ArchivedStyle::Alphanumeric,
        numeric: false,
        label: true,
        superscript: false,
    },
];

/// Find a style in the registry by alias (case-insensitive).
pub fn find_style_info(name: &str) -> Option<&'static StyleInfo> {
    let name_lower = name.to_lowercase();
    STYLE_REGISTRY
        .iter()
        .find(|info| info.aliases.iter().any(|&alias| alias == name_lower))
}

/// Get all supported style aliases (canonical names only).
pub fn supported_style_aliases() -> impl Iterator<Item = &'static str> {
    STYLE_REGISTRY.iter().map(|info| info.aliases[0])
}

/// Get all registry entries for detailed style information.
#[cfg(test)]
pub fn all_registry_styles() -> impl Iterator<Item = &'static StyleInfo> {
    STYLE_REGISTRY.iter()
}

/// Get the number of styles in the registry.
#[cfg(test)]
pub fn registry_style_count() -> usize {
    STYLE_REGISTRY.len()
}

/// Format a human-readable list of available styles grouped by format.
///
/// Returns a formatted string suitable for CLI help or documentation.
#[cfg(test)]
pub fn format_style_list() -> String {
    let mut numeric = Vec::new();
    let mut superscript = Vec::new();
    let mut author_date = Vec::new();

    for style in STYLE_REGISTRY {
        let name = style.aliases[0];
        if style.superscript {
            superscript.push(name);
        } else if style.numeric {
            numeric.push(name);
        } else {
            author_date.push(name);
        }
    }

    format!(
        "Numeric styles: {}\n\
         Superscript styles: {}\n\
         Author-date styles: {}",
        numeric.join(", "),
        superscript.join(", "),
        author_date.join(", ")
    )
}

/// Runtime-detected style characteristics for styles not in the registry.
///
/// Unlike `StyleInfo`, this is computed at runtime from CSL metadata.
/// Superscript detection is not possible from CSL alone, so it defaults to `false`.
#[derive(Debug, Clone, Copy)]
pub struct DetectedStyleFormat {
    /// Whether the style uses sequential numeric citations (from CSL category metadata)
    numeric: bool,
    /// Whether the style uses author-based labels (from CSL category metadata)
    label: bool,
    /// Whether the style uses superscript (cannot be detected from CSL, always false)
    superscript: bool,
}

impl CitationStyle for DetectedStyleFormat {
    fn is_numeric(&self) -> bool {
        self.numeric
    }
    fn is_label(&self) -> bool {
        self.label
    }
    fn is_superscript(&self) -> bool {
        self.superscript
    }
}

/// Detect citation format characteristics from a CSL style's metadata.
///
/// Examines the style's `info.category` to find the `CitationFormat` and determines
/// whether the style uses numeric or label citations. Superscript cannot be detected
/// from CSL metadata alone, so it always returns `false` for superscript.
///
/// # Arguments
/// * `style` - The loaded CSL IndependentStyle
///
/// # Returns
/// `DetectedStyleFormat` with numeric/label/superscript flags based on CSL metadata.
///
/// # Citation Format Mapping
/// - `CitationFormat::Numeric` → numeric=true (e.g., IEEE `[1]`)
/// - `CitationFormat::Label` → label=true (e.g., `[Smi24]`)
/// - `CitationFormat::AuthorDate` → neither (e.g., `(Smith, 2024)`)
/// - `CitationFormat::Author` → neither (e.g., `(Smith)`)
/// - `CitationFormat::Note` → neither (footnote styles)
pub fn detect_style_format(style: &IndependentStyle) -> DetectedStyleFormat {
    let citation_format = style.info.category.iter().find_map(|cat| match cat {
        StyleCategory::CitationFormat { format } => Some(*format),
        StyleCategory::Field { .. } => None,
    });

    let numeric = citation_format.is_some_and(|f| matches!(f, CitationFormat::Numeric));
    let label = citation_format.is_some_and(|f| matches!(f, CitationFormat::Label));

    DetectedStyleFormat {
        numeric,
        label,
        superscript: false, // Cannot detect from CSL metadata
    }
}
