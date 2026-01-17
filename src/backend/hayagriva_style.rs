//! Hayagriva CSL style registry and metadata.
//!
//! This module provides a centralized registry of supported citation styles
//! with their aliases and formatting characteristics (numeric vs author-date,
//! superscript vs bracketed).

use hayagriva::archive::ArchivedStyle;

/// Style metadata: single source of truth for aliases and citation format.
#[derive(Debug, Clone, Copy)]
pub struct StyleInfo {
    /// Short alias(es) for the style (first is canonical)
    pub aliases: &'static [&'static str],
    /// The hayagriva ArchivedStyle variant
    pub archived: ArchivedStyle,
    /// Whether this style uses numeric citations (e.g., [1], [2])
    pub numeric: bool,
    /// Whether this style uses superscript citations (e.g., Nature)
    pub superscript: bool,
}

/// Registry of supported style aliases with their metadata.
/// This is the single source of truth for style resolution and classification.
static STYLE_REGISTRY: &[StyleInfo] = &[
    StyleInfo {
        aliases: &["ieee"],
        archived: ArchivedStyle::InstituteOfElectricalAndElectronicsEngineers,
        numeric: true,
        superscript: false,
    },
    StyleInfo {
        aliases: &["apa", "american-psychological-association"],
        archived: ArchivedStyle::AmericanPsychologicalAssociation,
        numeric: false,
        superscript: false,
    },
    StyleInfo {
        aliases: &["chicago-author-date"],
        archived: ArchivedStyle::ChicagoAuthorDate,
        numeric: false,
        superscript: false,
    },
    StyleInfo {
        aliases: &["chicago-notes"],
        archived: ArchivedStyle::ChicagoNotes,
        numeric: false,
        superscript: false,
    },
    StyleInfo {
        aliases: &["mla", "modern-language-association"],
        archived: ArchivedStyle::ModernLanguageAssociation,
        numeric: false,
        superscript: false,
    },
    StyleInfo {
        aliases: &["mla8", "modern-language-association-8"],
        archived: ArchivedStyle::ModernLanguageAssociation8,
        numeric: false,
        superscript: false,
    },
    StyleInfo {
        aliases: &["nature"],
        archived: ArchivedStyle::Nature,
        numeric: true,
        superscript: true,
    },
    StyleInfo {
        aliases: &["vancouver"],
        archived: ArchivedStyle::Vancouver,
        numeric: true,
        superscript: false,
    },
    StyleInfo {
        aliases: &["vancouver-superscript"],
        archived: ArchivedStyle::VancouverSuperscript,
        numeric: true,
        superscript: true,
    },
    StyleInfo {
        aliases: &["harvard", "harvard-cite-them-right"],
        archived: ArchivedStyle::HarvardCiteThemRight,
        numeric: false,
        superscript: false,
    },
    StyleInfo {
        aliases: &["acm", "association-for-computing-machinery"],
        archived: ArchivedStyle::AssociationForComputingMachinery,
        numeric: true,
        superscript: false,
    },
    StyleInfo {
        aliases: &["acs", "american-chemical-society"],
        archived: ArchivedStyle::AmericanChemicalSociety,
        numeric: true,
        superscript: false,
    },
    StyleInfo {
        aliases: &["ama", "american-medical-association"],
        archived: ArchivedStyle::AmericanMedicalAssociation,
        numeric: true,
        superscript: false,
    },
    StyleInfo {
        aliases: &["springer-basic"],
        archived: ArchivedStyle::SpringerBasic,
        numeric: true,
        superscript: false,
    },
    StyleInfo {
        aliases: &["springer-basic-author-date"],
        archived: ArchivedStyle::SpringerBasicAuthorDate,
        numeric: false,
        superscript: false,
    },
    StyleInfo {
        aliases: &["cell"],
        archived: ArchivedStyle::Cell,
        numeric: true,
        superscript: false,
    },
    StyleInfo {
        aliases: &["elsevier-harvard"],
        archived: ArchivedStyle::ElsevierHarvard,
        numeric: false,
        superscript: false,
    },
    StyleInfo {
        aliases: &["elsevier-vancouver"],
        archived: ArchivedStyle::ElsevierVancouver,
        numeric: true,
        superscript: false,
    },
    StyleInfo {
        aliases: &["alphanumeric"],
        archived: ArchivedStyle::Alphanumeric,
        numeric: true,
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
