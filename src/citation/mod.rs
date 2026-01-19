use std::cell::RefCell;
use std::collections::HashSet;
use std::path::{Component, Path};

use indexmap::IndexMap;
use lazy_static::lazy_static;
use mdbook_preprocessor::book::{Book, BookItem, Chapter};
use regex::Regex;

use crate::backend::{BibliographyBackend, CitationContext, CitationVariant};
use crate::config::{CitationSyntax, SortOrder};
use crate::models::BibItem;
use crate::renderer;

static BIB_OUT_FILE: &str = "bibliography";

// Placeholder used to protect escaped @ symbols during processing.
// Uses Unicode private use area characters to avoid conflicts with normal text.
const ESCAPED_AT_PLACEHOLDER: &str = "\u{E000}MDBIB_ESCAPED_AT\u{E001}";
// Placeholder prefix for protected code blocks.
const CODE_BLOCK_PLACEHOLDER: &str = "\u{E000}MDBIB_CODEBLOCK";

// =============================================================================
// Citation Key Patterns
// =============================================================================
//
// IMPORTANT: The patterns below have intentionally different key formats:
//
// 1. mdbook-bib native patterns (REF_PATTERN, AT_REF_PATTERN):
//    - Key format: [a-zA-Z0-9_\-:./@]+
//    - Allows keys starting with digits (e.g., {{#cite 123key}}, @@2024smith)
//    - Follows BibLaTeX's permissive key rules
//
// 2. Pandoc-style patterns (PANDOC_*_PATTERN):
//    - Key format: [a-zA-Z_][a-zA-Z0-9_]* (must start with letter or underscore)
//    - Follows Pandoc's citation key specification for compatibility
//    - Keys like @123key will NOT match (use @key123 instead)
//
// This difference is intentional: Pandoc patterns match Pandoc's spec for
// cross-tool compatibility, while native patterns remain backward-compatible
// with existing mdbook-bib usage.
// =============================================================================

// Native mdbook-bib patterns
// BibLaTeX-compliant character class for citation keys:
// - Allowed: alphanumeric, underscore, hyphen, colon, dot, slash, at-symbol
// - Forbidden: spaces, comma, quotes, hash, braces, percent, tilde, parentheses, equals
pub const REF_PATTERN: &str = r"
(?x)                       # insignificant whitespace mode
\\\{\{\#.*\}\}               # match escaped placeholder
|                            # or
\{\{\s*                      # placeholder opening parens and whitespace
\#cite                       # explicitly match #cite (only, not other mdBook helpers like #include, #title)
\s+                          # separating whitespace
([a-zA-Z0-9_\-:./@]+)        # citation key (capture group 1) - BibLaTeX compliant, allows digit start
\s*\}\}                      # whitespace and placeholder closing parens";

pub const AT_REF_PATTERN: &str = r##"(@@)([a-zA-Z0-9_\-/@]+(?:[.:][a-zA-Z0-9_\-/@]+)*)"##;

// Pandoc-style citation patterns (only used when citation-syntax = "pandoc")
// These follow Pandoc's citation key spec: must start with letter or underscore.
// See: https://pandoc.org/MANUAL.html#citation-syntax

// Escaped @ - will be replaced with placeholder before processing
pub const ESCAPED_AT_PATTERN: &str = r"\\@";

// Pandoc bracketed with suppress-author: [-@key]
// Must be processed before regular bracketed to avoid partial matches
pub const PANDOC_SUPPRESS_AUTHOR_PATTERN: &str =
    r"\[-@([a-zA-Z_][a-zA-Z0-9_]*(?:[:.#$%&\-+?<>~/][a-zA-Z0-9_]+)*)\]";

// Pandoc bracketed: [@key]
pub const PANDOC_BRACKETED_PATTERN: &str =
    r"\[@([a-zA-Z_][a-zA-Z0-9_]*(?:[:.#$%&\-+?<>~/][a-zA-Z0-9_]+)*)\]";

// Pandoc author-in-text: @key (not preceded by \, @, word char, or /)
// Captures prefix char (group 1) to preserve it in replacement, key is in group 2.
// The / exclusion prevents matching @mentions in URLs like https://twitter.com/@user
// This is the most permissive pattern and must be processed last.
pub const PANDOC_CITE_PATTERN: &str =
    r"(^|[^\\@\w/])@([a-zA-Z_][a-zA-Z0-9_]*(?:[:.#$%&\-+?<>~/][a-zA-Z0-9_]+)*)";

// Code block patterns for protection
const FENCED_CODE_PATTERN: &str = r"(?s)```[^\n]*\n.*?```|~~~[^\n]*\n.*?~~~";
const INLINE_CODE_PATTERN: &str = r"`[^`\n]+`";

lazy_static! {
    static ref REF_REGEX: Regex = Regex::new(REF_PATTERN).unwrap();
    static ref AT_REF_REGEX: Regex = Regex::new(AT_REF_PATTERN).unwrap();
    // Pandoc patterns
    static ref ESCAPED_AT_REGEX: Regex = Regex::new(ESCAPED_AT_PATTERN).unwrap();
    static ref PANDOC_SUPPRESS_AUTHOR_REGEX: Regex =
        Regex::new(PANDOC_SUPPRESS_AUTHOR_PATTERN).unwrap();
    static ref PANDOC_BRACKETED_REGEX: Regex = Regex::new(PANDOC_BRACKETED_PATTERN).unwrap();
    static ref PANDOC_CITE_REGEX: Regex = Regex::new(PANDOC_CITE_PATTERN).unwrap();
    // Code block patterns
    static ref FENCED_CODE_REGEX: Regex = Regex::new(FENCED_CODE_PATTERN).unwrap();
    static ref INLINE_CODE_REGEX: Regex = Regex::new(INLINE_CODE_PATTERN).unwrap();
}

/// Result of citation expansion, containing both global and per-chapter citations.
pub struct CitationResult {
    /// All citations found across the entire book.
    pub all_cited: HashSet<String>,
    /// Citations found per chapter, keyed by chapter path.
    pub per_chapter: IndexMap<String, HashSet<String>>,
}

/// Protect code blocks from citation processing by replacing them with placeholders.
///
/// Returns the modified content and a vector of the original code blocks.
/// Code blocks can be restored later with `restore_code_blocks`.
fn protect_code_blocks(content: &str) -> (String, Vec<String>) {
    let mut blocks = Vec::new();
    let mut result = content.to_string();

    // First protect fenced code blocks (``` or ~~~)
    result = FENCED_CODE_REGEX
        .replace_all(&result, |caps: &regex::Captures| {
            let block = caps.get(0).unwrap().as_str().to_string();
            let idx = blocks.len();
            blocks.push(block);
            format!("{CODE_BLOCK_PLACEHOLDER}{idx}\u{E001}")
        })
        .into_owned();

    // Then protect inline code (`)
    result = INLINE_CODE_REGEX
        .replace_all(&result, |caps: &regex::Captures| {
            let block = caps.get(0).unwrap().as_str().to_string();
            let idx = blocks.len();
            blocks.push(block);
            format!("{CODE_BLOCK_PLACEHOLDER}{idx}\u{E001}")
        })
        .into_owned();

    (result, blocks)
}

/// Restore code blocks that were protected during citation processing.
fn restore_code_blocks(content: &str, blocks: &[String]) -> String {
    let mut result = content.to_string();
    for (idx, block) in blocks.iter().enumerate() {
        let placeholder = format!("{CODE_BLOCK_PLACEHOLDER}{idx}\u{E001}");
        // Use replacen to replace only the first occurrence, preventing cascade
        // if the code block content happens to contain the placeholder pattern
        result = result.replacen(&placeholder, block, 1);
    }
    result
}

/// Expand all citation references in the book, replacing placeholders with formatted citations.
pub fn expand_cite_references_in_book(
    book: &mut Book,
    bibliography: &mut IndexMap<String, BibItem>,
    backend: &dyn BibliographyBackend,
    citation_syntax: &CitationSyntax,
) -> CitationResult {
    let mut all_cited = HashSet::new();
    let mut per_chapter: IndexMap<String, HashSet<String>> = IndexMap::new();
    let mut last_index = 0;

    let syntax_info = match citation_syntax {
        CitationSyntax::Default => "{{#cite ...}} and @@citation",
        CitationSyntax::Pandoc => "{{#cite ...}}, @@citation, @key, [@key], and [-@key]",
    };

    book.for_each_mut(|section: &mut BookItem| {
        if let BookItem::Chapter(ref mut ch) = *section {
            if let Some(ref chapter_path) = ch.path {
                tracing::info!(
                    "Replacing placeholders: {} in {}",
                    syntax_info,
                    chapter_path.as_path().display()
                );
                let mut chapter_cited = HashSet::new();
                let new_content = replace_all_placeholders(
                    ch,
                    bibliography,
                    &mut chapter_cited,
                    backend,
                    &mut last_index,
                    citation_syntax,
                );
                ch.content = new_content;
                all_cited.extend(chapter_cited.clone());
                per_chapter.insert(chapter_path.display().to_string(), chapter_cited);
            }
        }
    });
    CitationResult {
        all_cited,
        per_chapter,
    }
}

/// Add bibliography at the end of each chapter.
pub fn add_bib_at_end_of_chapters(
    book: &mut Book,
    bibliography: &mut IndexMap<String, BibItem>,
    backend: &dyn BibliographyBackend,
    chapter_refs_header: &str,
    order: SortOrder,
    per_chapter_citations: &IndexMap<String, HashSet<String>>,
    css_html: &str,
) {
    book.for_each_mut(|section: &mut BookItem| {
        if let BookItem::Chapter(ref mut ch) = *section {
            if let Some(ref chapter_path) = ch.path {
                let chapter_key = chapter_path.display().to_string();
                let cited = per_chapter_citations
                    .get(&chapter_key)
                    .cloned()
                    .unwrap_or_default();

                if cited.is_empty() {
                    tracing::info!(
                        "No citations in chapter {}, skipping bibliography",
                        chapter_key
                    );
                    return;
                }

                tracing::info!("Adding bibliography at the end of chapter {}", chapter_key);
                tracing::info!("Refs cited in this chapter: {:?}", cited);

                let ch_bib_content_html = renderer::generate_bibliography_html(
                    bibliography,
                    &cited,
                    true,
                    backend,
                    order.clone(),
                );

                // Inject CSS at the start and bibliography at the end
                let new_content = format!(
                    "{}\n{}\n{}\n{}",
                    css_html, ch.content, chapter_refs_header, ch_bib_content_html
                );
                ch.content = new_content;
            }
        }
    });
}

/// Replace a single citation placeholder with its formatted citation.
///
/// This helper function handles the common logic for all citation patterns:
/// - Tracks the citation in the cited set
/// - Assigns an index if this is the first occurrence
/// - Formats the citation using the backend with the appropriate variant
/// - Returns appropriate error messages for missing or invalid citations
fn replace_citation_placeholder(
    citation_key: &str,
    chapter_path: &Path,
    bib: &RefCell<&mut IndexMap<String, BibItem>>,
    cited_set: &RefCell<&mut HashSet<String>>,
    idx: &RefCell<&mut u32>,
    backend: &dyn BibliographyBackend,
    variant: CitationVariant,
) -> String {
    let cite = citation_key.trim();

    // Track this citation
    cited_set.borrow_mut().insert(cite.to_owned());

    let mut bib_mut = bib.borrow_mut();
    let mut idx_mut = idx.borrow_mut();

    if bib_mut.contains_key(cite) {
        let path_to_root = breadcrumbs_up_to_root(chapter_path);
        let item = bib_mut.get_mut(cite).unwrap();

        // Assign index on first occurrence
        if item.index.is_none() {
            **idx_mut += 1;
            item.index = Some(**idx_mut);
        }

        let context = CitationContext {
            bib_page_path: format!("{path_to_root}{BIB_OUT_FILE}.html"),
            chapter_path: chapter_path.display().to_string(),
            variant,
        };

        let formatted = backend.format_citation(item, &context).unwrap_or_else(|e| {
            tracing::error!("Failed to format citation for '{}': {}", cite, e);
            format!("\\[Error formatting {cite}\\]")
        });

        tracing::info!(
            "Citation replacement ({:?}): '{}' -> '{}'",
            variant,
            cite,
            formatted
        );
        formatted
    } else {
        tracing::warn!("Unknown bibliography reference: '{}'", cite);
        format!("\\[Unknown bib ref: {cite}\\]")
    }
}

pub fn replace_all_placeholders(
    chapter: &Chapter,
    bibliography: &mut IndexMap<String, BibItem>,
    cited: &mut HashSet<String>,
    backend: &dyn BibliographyBackend,
    last_index: &mut u32,
    citation_syntax: &CitationSyntax,
) -> String {
    let chapter_path = chapter.path.as_deref().unwrap_or_else(|| Path::new(""));

    // Wrap mutable state in RefCell for interior mutability
    let bib = RefCell::new(bibliography);
    let cited_set = RefCell::new(cited);
    let idx = RefCell::new(last_index);

    // Step 1: Protect code blocks from citation processing
    let (mut content, code_blocks) = protect_code_blocks(&chapter.content);

    // Step 2: For Pandoc syntax, protect escaped @ symbols
    if *citation_syntax == CitationSyntax::Pandoc {
        content = ESCAPED_AT_REGEX
            .replace_all(&content, ESCAPED_AT_PLACEHOLDER)
            .into_owned();
    }

    // Step 3: Replace all {{#cite ...}} patterns (capture group 1)
    content = REF_REGEX
        .replace_all(&content, |caps: &regex::Captures| {
            let citation_key = caps.get(1).map(|m| m.as_str()).unwrap_or("");
            replace_citation_placeholder(
                citation_key,
                chapter_path,
                &bib,
                &cited_set,
                &idx,
                backend,
                CitationVariant::Standard,
            )
        })
        .into_owned();

    // Step 4: Replace all @@cite patterns (capture group 2)
    // Must be done before single @ patterns to avoid partial matches
    content = AT_REF_REGEX
        .replace_all(&content, |caps: &regex::Captures| {
            let citation_key = caps.get(2).map(|m| m.as_str()).unwrap_or("");
            replace_citation_placeholder(
                citation_key,
                chapter_path,
                &bib,
                &cited_set,
                &idx,
                backend,
                CitationVariant::Standard,
            )
        })
        .into_owned();

    // Step 5: Process Pandoc-style patterns if enabled
    if *citation_syntax == CitationSyntax::Pandoc {
        // Process [-@key] (suppress author) - must come before [@key]
        content = PANDOC_SUPPRESS_AUTHOR_REGEX
            .replace_all(&content, |caps: &regex::Captures| {
                let citation_key = caps.get(1).map(|m| m.as_str()).unwrap_or("");
                replace_citation_placeholder(
                    citation_key,
                    chapter_path,
                    &bib,
                    &cited_set,
                    &idx,
                    backend,
                    CitationVariant::SuppressAuthor,
                )
            })
            .into_owned();

        // Process [@key] (parenthetical)
        content = PANDOC_BRACKETED_REGEX
            .replace_all(&content, |caps: &regex::Captures| {
                let citation_key = caps.get(1).map(|m| m.as_str()).unwrap_or("");
                replace_citation_placeholder(
                    citation_key,
                    chapter_path,
                    &bib,
                    &cited_set,
                    &idx,
                    backend,
                    CitationVariant::Parenthetical,
                )
            })
            .into_owned();

        // Process @key (author-in-text) - must be last as it's most permissive
        // Pattern captures prefix char (group 1) to avoid matching emails; key is in group 2
        content = PANDOC_CITE_REGEX
            .replace_all(&content, |caps: &regex::Captures| {
                let prefix = caps.get(1).map(|m| m.as_str()).unwrap_or("");
                let citation_key = caps.get(2).map(|m| m.as_str()).unwrap_or("");
                let replacement = replace_citation_placeholder(
                    citation_key,
                    chapter_path,
                    &bib,
                    &cited_set,
                    &idx,
                    backend,
                    CitationVariant::AuthorInText,
                );
                format!("{prefix}{replacement}")
            })
            .into_owned();

        // Restore escaped @ symbols
        content = content.replace(ESCAPED_AT_PLACEHOLDER, "@");
    }

    // Step 6: Restore code blocks
    restore_code_blocks(&content, &code_blocks)
}

fn breadcrumbs_up_to_root(source_file: &Path) -> String {
    if source_file.as_os_str().is_empty() {
        return String::new();
    }

    let components_count = source_file.components().fold(0, |acc, c| match c {
        Component::Normal(_) => acc + 1,
        Component::ParentDir => acc - 1,
        Component::CurDir => acc,
        Component::RootDir | Component::Prefix(_) => panic!(
            "mdBook is not supposed to give us absolute paths, only relative from the book root."
        ),
    }) - 1;

    let mut to_root = vec![".."; components_count].join("/");
    if components_count > 0 {
        to_root.push('/');
    }

    to_root
}
