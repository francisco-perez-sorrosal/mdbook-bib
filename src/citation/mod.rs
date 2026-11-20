use std::cell::RefCell;
use std::collections::HashSet;
use std::path::{Component, Path};

use indexmap::IndexMap;
use lazy_static::lazy_static;
use mdbook_preprocessor::book::{Book, BookItem, Chapter};
use regex::Regex;

use hayagriva::citationberg::taxonomy::Locator;

use crate::backend::{
    BibliographyBackend, CitationContext, CitationGroupItem, CitationLocator, CitationVariant,
};
use crate::config::{CitationSyntax, SortOrder};
use crate::models::BibItem;
use crate::renderer;

static BIB_OUT_FILE: &str = "bibliography";

// Placeholder used to protect escaped @ symbols during processing.
// Uses Unicode private use area characters to avoid conflicts with normal text.
const ESCAPED_AT_PLACEHOLDER: &str = "\u{E000}MDBIB_ESCAPED_AT\u{E001}";
// Placeholder prefix for protected code blocks.
const CODE_BLOCK_PLACEHOLDER: &str = "\u{E000}MDBIB_CODEBLOCK";
// Placeholder prefix for protected markdown links during Pandoc processing.
const MD_LINK_PLACEHOLDER: &str = "\u{E000}MDBIB_MDLINK";

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
pub const REF_PATTERN: &str = r"(?x) # enable insignificant whitespace mode + in-regex comments
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

// Pandoc bracketed with suppress-author: [-@key] (kept for test/API compatibility)
#[allow(dead_code)]
pub const PANDOC_SUPPRESS_AUTHOR_PATTERN: &str =
    r"\[-@([a-zA-Z_][a-zA-Z0-9_]*(?:[:.#$%&\-+?<>~/][a-zA-Z0-9_]+)*)\]";

// Pandoc bracketed: [@key] (kept for test/API compatibility)
#[allow(dead_code)]
pub const PANDOC_BRACKETED_PATTERN: &str =
    r"\[@([a-zA-Z_][a-zA-Z0-9_]*(?:[:.#$%&\-+?<>~/][a-zA-Z0-9_]+)*)\]";

// Pandoc citation key pattern inside a bracketed segment: optional `-` then `@key`
const PANDOC_SEGMENT_KEY_PATTERN: &str =
    r"(-?)@([a-zA-Z_][a-zA-Z0-9_]*(?:[:.#$%&\-+?<>~/][a-zA-Z0-9_]+)*)";

// Pandoc author-in-text: @key (not preceded by \, @, word char, or /)
// Captures prefix char (group 1) to preserve it in replacement, key is in group 2.
// The / exclusion prevents matching @mentions in URLs like https://twitter.com/@user
// This is the most permissive pattern and must be processed last.
pub const PANDOC_CITE_PATTERN: &str =
    r"(^|[^\\@\w/])@([a-zA-Z_][a-zA-Z0-9_]*(?:[:.#$%&\-+?<>~/][a-zA-Z0-9_]+)*)";

// Code block patterns for protection
const FENCED_CODE_PATTERN: &str = r"(?s)```[^\n]*\n.*?```|~~~[^\n]*\n.*?~~~";
const INLINE_CODE_PATTERN: &str = r"`[^`\n]+`";
const MARKDOWN_LINK_PATTERN: &str = r"\[[^\[\]\n]+\]\([^\)\n]+\)";

lazy_static! {
    static ref REF_REGEX: Regex = Regex::new(REF_PATTERN).unwrap();
    static ref AT_REF_REGEX: Regex = Regex::new(AT_REF_PATTERN).unwrap();
    // Pandoc patterns
    static ref ESCAPED_AT_REGEX: Regex = Regex::new(ESCAPED_AT_PATTERN).unwrap();
    static ref PANDOC_SEGMENT_KEY_REGEX: Regex = Regex::new(PANDOC_SEGMENT_KEY_PATTERN).unwrap();
    static ref PANDOC_CITE_REGEX: Regex = Regex::new(PANDOC_CITE_PATTERN).unwrap();
    // Code block patterns
    static ref FENCED_CODE_REGEX: Regex = Regex::new(FENCED_CODE_PATTERN).unwrap();
    static ref INLINE_CODE_REGEX: Regex = Regex::new(INLINE_CODE_PATTERN).unwrap();
    static ref MARKDOWN_LINK_REGEX: Regex = Regex::new(MARKDOWN_LINK_PATTERN).unwrap();
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

/// Protect markdown links `[text](url)` from Pandoc citation processing.
fn protect_markdown_links(content: &str) -> (String, Vec<String>) {
    let mut links = Vec::new();
    let result = MARKDOWN_LINK_REGEX
        .replace_all(content, |caps: &regex::Captures| {
            let link = caps.get(0).unwrap().as_str().to_string();
            let idx = links.len();
            links.push(link);
            format!("{MD_LINK_PLACEHOLDER}{idx}\u{E001}")
        })
        .into_owned();
    (result, links)
}

/// Restore markdown links protected during Pandoc citation processing.
fn restore_markdown_links(content: &str, links: &[String]) -> String {
    let mut result = content.to_string();
    for (idx, link) in links.iter().enumerate() {
        let placeholder = format!("{MD_LINK_PLACEHOLDER}{idx}\u{E001}");
        result = result.replacen(&placeholder, link, 1);
    }
    result
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
                tracing::debug!(
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
                    tracing::debug!(
                        "No citations in chapter {}, skipping bibliography",
                        chapter_key
                    );
                    return;
                }

                tracing::debug!("Adding bibliography at the end of chapter {}", chapter_key);
                tracing::debug!("Refs cited in this chapter: {:?}", cited);

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

/// Parsed item from a Pandoc bracketed citation group (`[prefix -@key, locator suffix]`).
#[derive(Debug, Clone, PartialEq, Eq)]
struct ParsedPandocItem {
    prefix: Option<String>,
    key: String,
    suppress_author: bool,
    locator: Option<CitationLocator>,
    suffix: Option<String>,
    raw_locator: Option<String>,
}

const LOCATOR_TERMS: &[(&str, Locator)] = &[
    ("sub verbo", Locator::SubVerbo),
    ("sub-verbo", Locator::SubVerbo),
    ("paragraphs", Locator::Paragraph),
    ("paragraph", Locator::Paragraph),
    ("appendices", Locator::Appendix),
    ("appendix", Locator::Appendix),
    ("chapters", Locator::Chapter),
    ("chapter", Locator::Chapter),
    ("sections", Locator::Section),
    ("section", Locator::Section),
    ("supplements", Locator::Supplement),
    ("supplement", Locator::Supplement),
    ("equations", Locator::Equation),
    ("equation", Locator::Equation),
    ("figures", Locator::Figure),
    ("figure", Locator::Figure),
    ("volumes", Locator::Volume),
    ("volume", Locator::Volume),
    ("columns", Locator::Column),
    ("column", Locator::Column),
    ("folios", Locator::Folio),
    ("folio", Locator::Folio),
    ("issues", Locator::Issue),
    ("issue", Locator::Issue),
    ("lines", Locator::Line),
    ("line", Locator::Line),
    ("notes", Locator::Note),
    ("note", Locator::Note),
    ("pages", Locator::Page),
    ("page", Locator::Page),
    ("parts", Locator::Part),
    ("part", Locator::Part),
    ("scenes", Locator::Scene),
    ("scene", Locator::Scene),
    ("tables", Locator::Table),
    ("table", Locator::Table),
    ("verses", Locator::Verse),
    ("verse", Locator::Verse),
    ("books", Locator::Book),
    ("book", Locator::Book),
    ("chaps", Locator::Chapter),
    ("chap", Locator::Chapter),
    ("paras", Locator::Paragraph),
    ("para", Locator::Paragraph),
    ("secs", Locator::Section),
    ("sec", Locator::Section),
    ("figs", Locator::Figure),
    ("fig", Locator::Figure),
    ("vols", Locator::Volume),
    ("vol", Locator::Volume),
    ("apps", Locator::Appendix),
    ("app", Locator::Appendix),
    ("cols", Locator::Column),
    ("col", Locator::Column),
    ("fols", Locator::Folio),
    ("fol", Locator::Folio),
    ("opera", Locator::Opus),
    ("opus", Locator::Opus),
    ("supp", Locator::Supplement),
    ("canon", Locator::Canon),
    ("title", Locator::Title),
    ("rule", Locator::Rule),
    ("s.v.", Locator::SubVerbo),
    ("s.v", Locator::SubVerbo),
    ("bks", Locator::Book),
    ("bk", Locator::Book),
    ("eqs", Locator::Equation),
    ("eq", Locator::Equation),
    ("nos", Locator::Issue),
    ("no", Locator::Issue),
    ("opp", Locator::Opus),
    ("op", Locator::Opus),
    ("pts", Locator::Part),
    ("pt", Locator::Part),
    ("tbl", Locator::Table),
    ("can", Locator::Canon),
    ("act", Locator::Act),
    ("ch", Locator::Chapter),
    ("pp", Locator::Page),
    ("ll", Locator::Line),
    ("nn", Locator::Note),
    ("vv", Locator::Verse),
    ("sc", Locator::Scene),
    ("§§", Locator::Section),
    ("¶¶", Locator::Paragraph),
    ("§", Locator::Section),
    ("¶", Locator::Paragraph),
    ("p", Locator::Page),
    ("l", Locator::Line),
    ("n", Locator::Note),
    ("v", Locator::Verse),
];

/// Split the portion after a locator term (e.g., `"12"`, `"3-5, and passim"`, `"12 and ff."`)
/// into `(locator_value, optional_suffix)`.
fn split_locator_value_and_suffix(input: &str) -> Option<(String, Option<String>)> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return None;
    }

    // Split off comma-introduced suffix unless the comma precedes another page number
    let (head, comma_tail) = if let Some(comma_idx) = trimmed.find(',') {
        let after_comma = trimmed[comma_idx + 1..].trim_start();
        if after_comma
            .chars()
            .next()
            .is_some_and(|c| c.is_ascii_digit())
        {
            (trimmed, None)
        } else {
            (&trimmed[..comma_idx], Some(&trimmed[comma_idx..]))
        }
    } else {
        (trimmed, None)
    };

    let head_trimmed = head.trim();
    if head_trimmed.is_empty() {
        return None;
    }

    // If there are multiple whitespace-separated tokens, keep the first token as the locator
    // value and preserve trailing words as suffix (unless it's a numeric comma list like "12, 15").
    let (value, word_tail) = if comma_tail.is_none() && head_trimmed.contains(',') {
        (head_trimmed, "")
    } else if let Some(ws_idx) = head_trimmed.find(char::is_whitespace) {
        (&head_trimmed[..ws_idx], &head_trimmed[ws_idx..])
    } else {
        (head_trimmed, "")
    };

    if value.is_empty() {
        return None;
    }

    let mut suffix = String::new();
    suffix.push_str(word_tail);
    if let Some(tail) = comma_tail {
        suffix.push_str(tail);
    }

    Some((
        value.to_string(),
        if suffix.trim().is_empty() {
            None
        } else {
            Some(suffix)
        },
    ))
}

/// Match a known Pandoc locator label at the start of `after_sep` (case-insensitive).
fn match_locator_term(after_sep: &str) -> Option<(Locator, &str)> {
    let lower = after_sep.to_lowercase();
    for &(term, locator) in LOCATOR_TERMS {
        if !lower.starts_with(term) {
            continue;
        }
        // Check boundary in original string by matching byte length of term
        // (for ASCII terms lower and original have identical byte lengths;
        // for '§'/'¶' both are identical UTF-8 sequences).
        let after_term = after_sep.get(term.len()..)?;
        if term.ends_with('.') {
            if after_term.starts_with(char::is_whitespace) {
                return Some((locator, after_term.trim_start()));
            }
        } else if let Some(after_dot) = after_term.strip_prefix('.') {
            if after_dot.starts_with(char::is_whitespace)
                || after_dot.chars().next().is_some_and(|c| c.is_ascii_digit())
            {
                return Some((locator, after_dot.trim_start()));
            }
        } else if after_term.starts_with(char::is_whitespace) {
            return Some((locator, after_term.trim_start()));
        }
    }
    None
}

/// Parse the remainder after `@key` inside a bracketed Pandoc citation segment.
fn parse_locator_and_suffix(
    remainder: &str,
) -> (Option<CitationLocator>, Option<String>, Option<String>) {
    if remainder.trim().is_empty() {
        return (None, None, None);
    }

    let raw_locator = Some(remainder.to_string());
    let trimmed_start = remainder.trim_start();
    let (had_comma, after_sep) = if let Some(rest) = trimmed_start.strip_prefix(',') {
        (true, rest.trim_start())
    } else {
        (false, trimmed_start)
    };

    if after_sep.is_empty() {
        return (None, Some(remainder.to_string()), raw_locator);
    }

    if let Some((kind, val_rest)) = match_locator_term(after_sep) {
        if let Some((value, suffix)) = split_locator_value_and_suffix(val_rest) {
            return (Some(CitationLocator { kind, value }), suffix, raw_locator);
        }
    } else if had_comma && after_sep.chars().next().is_some_and(|c| c.is_ascii_digit()) {
        if let Some((value, suffix)) = split_locator_value_and_suffix(after_sep) {
            return (
                Some(CitationLocator {
                    kind: Locator::Page,
                    value,
                }),
                suffix,
                raw_locator,
            );
        }
    }

    (None, Some(remainder.to_string()), raw_locator)
}

/// Parse the interior of `[...]` as one or more `;`-separated Pandoc citation items.
/// Returns `None` if any segment does not contain a valid Pandoc `@key`/`-@key`.
fn parse_pandoc_bracketed_group(inner: &str) -> Option<Vec<ParsedPandocItem>> {
    if inner.contains('[') || inner.contains(']') || inner.contains("\n\n") {
        return None;
    }

    let mut items = Vec::new();
    for segment in inner.split(';') {
        let seg = segment.trim_start();
        let mut matched = None;
        for caps in PANDOC_SEGMENT_KEY_REGEX.captures_iter(seg) {
            let full = caps.get(0).unwrap();
            // Verify boundary before `-@key` or `@key` to avoid matching emails/URLs/@@
            let valid_boundary = if full.start() == 0 {
                true
            } else {
                seg[..full.start()].chars().next_back().is_some_and(|prev| {
                    !prev.is_alphanumeric()
                        && prev != '_'
                        && prev != '\\'
                        && prev != '/'
                        && prev != '@'
                })
            };
            if valid_boundary {
                matched = Some(caps);
                break;
            }
        }

        let caps = matched?;
        let full = caps.get(0).unwrap();
        let suppress_author = !caps.get(1).map(|m| m.as_str()).unwrap_or("").is_empty();
        let key = caps.get(2).unwrap().as_str().to_string();

        let prefix_slice = &seg[..full.start()];
        let prefix = if prefix_slice.is_empty() {
            None
        } else {
            Some(prefix_slice.to_string())
        };

        let remainder = &seg[full.end()..];
        let (locator, suffix, raw_locator) = parse_locator_and_suffix(remainder);

        items.push(ParsedPandocItem {
            prefix,
            key,
            suppress_author,
            locator,
            suffix,
            raw_locator,
        });
    }

    if items.is_empty() {
        None
    } else {
        Some(items)
    }
}

/// Replace a parsed bracketed citation group using the bibliography backend.
fn replace_citation_group_placeholder(
    parsed_items: &[ParsedPandocItem],
    chapter_path: &Path,
    bib: &RefCell<&mut IndexMap<String, BibItem>>,
    cited_set: &RefCell<&mut HashSet<String>>,
    idx: &RefCell<&mut u32>,
    backend: &dyn BibliographyBackend,
) -> String {
    // Track all keys and assign indices on first occurrence
    {
        let mut cited_mut = cited_set.borrow_mut();
        let mut bib_mut = bib.borrow_mut();
        let mut idx_mut = idx.borrow_mut();

        for parsed in parsed_items {
            let cite = parsed.key.trim();
            cited_mut.insert(cite.to_owned());
            if let Some(item) = bib_mut.get_mut(cite) {
                if item.index.is_none() {
                    **idx_mut += 1;
                    item.index = Some(**idx_mut);
                }
            }
        }
    }

    let bib_ref = bib.borrow();
    let path_to_root = breadcrumbs_up_to_root(chapter_path);
    let context = CitationContext {
        bib_page_path: format!("{path_to_root}{BIB_OUT_FILE}.html"),
        chapter_path: chapter_path.display().to_string(),
        variant: CitationVariant::Parenthetical,
    };

    // Fast path: all keys exist in bibliography
    let all_known = parsed_items
        .iter()
        .all(|p| bib_ref.contains_key(p.key.trim()));

    if all_known {
        let group_items: Vec<CitationGroupItem<'_>> = parsed_items
            .iter()
            .map(|p| CitationGroupItem {
                item: bib_ref.get(p.key.trim()).unwrap(),
                prefix: p.prefix.clone(),
                locator: p.locator.clone(),
                suffix: p.suffix.clone(),
                suppress_author: p.suppress_author,
                raw_locator: p.raw_locator.clone(),
            })
            .collect();

        return backend
            .format_citation_group(&group_items, &context)
            .unwrap_or_else(|e| {
                tracing::error!("Failed to format citation group: {}", e);
                let keys: Vec<&str> = parsed_items.iter().map(|p| p.key.as_str()).collect();
                format!("\\[Error formatting {}\\]", keys.join("; "))
            });
    }

    // Handle single or mixed unknown keys gracefully
    let mut parts = Vec::with_capacity(parsed_items.len());
    for p in parsed_items {
        let cite = p.key.trim();
        if let Some(item) = bib_ref.get(cite) {
            let single = CitationGroupItem {
                item,
                prefix: p.prefix.clone(),
                locator: p.locator.clone(),
                suffix: p.suffix.clone(),
                suppress_author: p.suppress_author,
                raw_locator: p.raw_locator.clone(),
            };
            parts.push(
                backend
                    .format_citation_group(&[single], &context)
                    .unwrap_or_else(|_| format!("\\[Error formatting {cite}\\]")),
            );
        } else {
            tracing::warn!("Unknown bibliography reference: '{}'", cite);
            parts.push(format!("\\[Unknown bib ref: {cite}\\]"));
        }
    }
    parts.join("; ")
}

/// Scan `content` for Pandoc bracketed citation groups (`[...]`) and replace them.
fn replace_pandoc_bracketed_groups(
    content: &str,
    chapter_path: &Path,
    bib: &RefCell<&mut IndexMap<String, BibItem>>,
    cited_set: &RefCell<&mut HashSet<String>>,
    idx: &RefCell<&mut u32>,
    backend: &dyn BibliographyBackend,
) -> String {
    let mut out = String::with_capacity(content.len());
    let mut pos = 0;
    let bytes = content.as_bytes();

    while pos < bytes.len() {
        let Some(rel_open) = content[pos..].find('[') else {
            out.push_str(&content[pos..]);
            break;
        };
        let open = pos + rel_open;
        out.push_str(&content[pos..open]);

        // Scan forward for matching ']', aborting if a nested '[' or blank line occurs
        let mut scan = open + 1;
        let mut close_idx = None;
        let mut nested_open = None;
        let mut blank_line = false;

        while scan < bytes.len() {
            let b = bytes[scan];
            if b == b'[' {
                nested_open = Some(scan);
                break;
            }
            if b == b']' {
                close_idx = Some(scan);
                break;
            }
            if b == b'\n'
                && (bytes.get(scan + 1) == Some(&b'\n')
                    || (bytes.get(scan + 1) == Some(&b'\r') && bytes.get(scan + 2) == Some(&b'\n')))
            {
                blank_line = true;
                break;
            }
            scan += 1;
        }

        if let Some(nested) = nested_open {
            // Copy up to the nested '[' and resume from there so inner [@cite] still works
            out.push_str(&content[open..nested]);
            pos = nested;
            continue;
        }

        if blank_line || close_idx.is_none() {
            out.push('[');
            pos = open + 1;
            continue;
        }

        let close = close_idx.unwrap();
        // Skip markdown links [text](url) and reference links [text][ref]
        let next_byte = bytes.get(close + 1).copied();
        if matches!(next_byte, Some(b'(') | Some(b'[')) {
            out.push_str(&content[open..=close]);
            pos = close + 1;
            continue;
        }

        let inner = &content[open + 1..close];
        if inner.contains('@') {
            if let Some(parsed) = parse_pandoc_bracketed_group(inner) {
                let replacement = replace_citation_group_placeholder(
                    &parsed,
                    chapter_path,
                    bib,
                    cited_set,
                    idx,
                    backend,
                );
                out.push_str(&replacement);
                pos = close + 1;
                continue;
            }
        }

        out.push_str(&content[open..=close]);
        pos = close + 1;
    }

    out
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

        tracing::debug!(
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
        // Protect markdown links `[text](url)` so `@` inside link text/URLs is not expanded
        let (protected, md_links) = protect_markdown_links(&content);
        content = protected;

        // Process bracketed citations: [@key], [-@key], [@a; @b], [@a, p. 12], [see @a, p. 12; @b]
        content = replace_pandoc_bracketed_groups(
            &content,
            chapter_path,
            &bib,
            &cited_set,
            &idx,
            backend,
        );

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

        // Restore markdown links and escaped @ symbols
        content = restore_markdown_links(&content, &md_links);
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
