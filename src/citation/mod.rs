use std::cell::RefCell;
use std::collections::HashSet;
use std::path::{Component, Path};

use indexmap::IndexMap;
use lazy_static::lazy_static;
use mdbook_preprocessor::book::{Book, BookItem, Chapter};
use regex::Regex;

use crate::backend::{BibliographyBackend, CitationContext};
use crate::config::SortOrder;
use crate::models::BibItem;
use crate::renderer;

static BIB_OUT_FILE: &str = "bibliography";

// Regex patterns for citation placeholders
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
([a-zA-Z0-9_\-:./@]+)        # citation key (capture group 1) - BibLaTeX compliant
\s*\}\}                      # whitespace and placeholder closing parens";

pub const AT_REF_PATTERN: &str = r##"(@@)([a-zA-Z0-9_\-/@]+(?:[.:][a-zA-Z0-9_\-/@]+)*)"##;

lazy_static! {
    static ref REF_REGEX: Regex = Regex::new(REF_PATTERN).unwrap();
    static ref AT_REF_REGEX: Regex = Regex::new(AT_REF_PATTERN).unwrap();
}

/// Expand all citation references in the book, replacing placeholders with formatted citations.
pub fn expand_cite_references_in_book(
    book: &mut Book,
    bibliography: &mut IndexMap<String, BibItem>,
    backend: &dyn BibliographyBackend,
) -> HashSet<String> {
    let mut cited = HashSet::new();
    let mut last_index = 0;
    book.for_each_mut(|section: &mut BookItem| {
        if let BookItem::Chapter(ref mut ch) = *section {
            if let Some(ref chapter_path) = ch.path {
                tracing::info!(
                    "Replacing placeholders: {{{{#cite ...}}}} and @@citation in {}",
                    chapter_path.as_path().display()
                );
                let new_content = replace_all_placeholders(
                    ch,
                    bibliography,
                    &mut cited,
                    backend,
                    &mut last_index,
                );
                ch.content = new_content;
            }
        }
    });
    cited
}

/// Add bibliography at the end of each chapter.
pub fn add_bib_at_end_of_chapters(
    book: &mut Book,
    bibliography: &mut IndexMap<String, BibItem>,
    backend: &dyn BibliographyBackend,
    chapter_refs_header: &str,
    order: SortOrder,
) {
    book.for_each_mut(|section: &mut BookItem| {
        if let BookItem::Chapter(ref mut ch) = *section {
            if let Some(ref chapter_path) = ch.path {
                tracing::info!(
                    "Adding bibliography at the end of chapter {}",
                    chapter_path.as_path().display()
                );

                let mut cited = HashSet::new();
                // Find all {{#cite ...}} keys
                for caps in REF_REGEX.captures_iter(&ch.content) {
                    if let Some(cite) = caps.get(1) {
                        cited.insert(cite.as_str().trim().to_owned());
                    }
                }
                // Find all @@... keys
                for caps in AT_REF_REGEX.captures_iter(&ch.content) {
                    if let Some(cite) = caps.get(2) {
                        cited.insert(cite.as_str().trim().to_owned());
                    }
                }
                tracing::info!("Refs cited in this chapter: {:?}", cited);

                let ch_bib_content_html = renderer::generate_bibliography_html(
                    bibliography,
                    &cited,
                    true,
                    backend,
                    order.clone(),
                );

                let new_content = String::from(ch.content.as_str())
                    + chapter_refs_header
                    + ch_bib_content_html.as_str();
                ch.content = new_content;
            }
        }
    });
}

pub fn replace_all_placeholders(
    chapter: &Chapter,
    bibliography: &mut IndexMap<String, BibItem>,
    cited: &mut HashSet<String>,
    backend: &dyn BibliographyBackend,
    last_index: &mut u32,
) -> String {
    let chapter_path = chapter.path.as_deref().unwrap_or_else(|| Path::new(""));

    // Wrap mutable state in RefCell for interior mutability
    let bib = RefCell::new(bibliography);
    let cited_set = RefCell::new(cited);
    let idx = RefCell::new(last_index);

    // First replace all {{#cite ...}}
    let replaced = REF_REGEX.replace_all(&chapter.content, |caps: &regex::Captures| {
        let cite = caps.get(1).map(|m| m.as_str().trim()).unwrap_or("");
        cited_set.borrow_mut().insert(cite.to_owned());
        let mut bib_mut = bib.borrow_mut();
        let mut idx_mut = idx.borrow_mut();
        if bib_mut.contains_key(cite) {
            let path_to_root = breadcrumbs_up_to_root(chapter_path);
            let item = bib_mut.get_mut(cite).unwrap();
            if item.index.is_none() {
                **idx_mut += 1;
                item.index = Some(**idx_mut);
            }
            let context = CitationContext {
                bib_page_path: format!("{path_to_root}{BIB_OUT_FILE}.html"),
                chapter_path: chapter_path.display().to_string(),
            };
            backend.format_citation(item, &context).unwrap_or_else(|e| {
                tracing::error!("Failed to format citation for '{}': {}", cite, e);
                format!("\\[Error formatting {cite}\\]")
            })
        } else {
            format!("\\[Unknown bib ref: {cite}\\]")
        }
    });

    // Then replace all @@cite
    let replaced = AT_REF_REGEX.replace_all(&replaced, |caps: &regex::Captures| {
        let cite = caps.get(2).map(|m| m.as_str().trim()).unwrap_or("");
        cited_set.borrow_mut().insert(cite.to_owned());
        let mut bib_mut = bib.borrow_mut();
        let mut idx_mut = idx.borrow_mut();
        if bib_mut.contains_key(cite) {
            let path_to_root = breadcrumbs_up_to_root(chapter_path);
            let item = bib_mut.get_mut(cite).unwrap();
            if item.index.is_none() {
                **idx_mut += 1;
                item.index = Some(**idx_mut);
            }
            let context = CitationContext {
                bib_page_path: format!("{path_to_root}{BIB_OUT_FILE}.html"),
                chapter_path: chapter_path.display().to_string(),
            };
            backend.format_citation(item, &context).unwrap_or_else(|e| {
                tracing::error!("Failed to format citation for '{}': {}", cite, e);
                format!("\\[Error formatting {cite}\\]")
            })
        } else {
            format!("\\[Unknown bib ref: {cite}\\]")
        }
    });

    replaced.into_owned()
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
