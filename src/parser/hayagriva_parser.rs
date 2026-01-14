use anyhow::Context;
use hayagriva::io::{from_biblatex_str, from_yaml_str};
use hayagriva::types::Person;
use indexmap::IndexMap;
use mdbook_preprocessor::errors::{Error, Result as MdResult};
use std::sync::Arc;

use crate::models::BibItem;

/// Parse bibliography content using hayagriva.
/// Supports both BibTeX/BibLaTeX and YAML formats.
pub fn parse_bibliography(
    raw_content: String,
    format: BibFormat,
) -> MdResult<IndexMap<String, BibItem>, Error> {
    tracing::info!(
        "🦀 HAYAGRIVA PARSER: Parsing bibliography (format: {:?})...",
        format
    );

    let bibliography = match format {
        BibFormat::BibTeX => from_biblatex_str(&raw_content)
            .map_err(|e| anyhow::anyhow!("Failed to parse BibTeX/BibLaTeX content: {e:?}"))?,
        BibFormat::Yaml => {
            from_yaml_str(&raw_content).context("Failed to parse YAML bibliography")?
        }
    };

    tracing::info!("{} bibliography items read", bibliography.len());

    let result: IndexMap<String, BibItem> = bibliography
        .iter()
        .map(|entry| {
            let citation_key = entry.key().to_string();
            tracing::info!("Processing bibliography entry: {}", citation_key);

            let title = entry.title().map(format_string_to_text).unwrap_or_else(|| {
                tracing::warn!(
                    "Entry {}: missing title field, using 'Not Found'",
                    citation_key
                );
                "Not Found".to_string()
            });

            let authors = extract_authors(entry, &citation_key);
            let summary = extract_summary(entry, &citation_key);
            let url = extract_url(entry, &citation_key);
            let (pub_year, pub_month) = extract_date(entry, &citation_key);

            // Extract extended fields
            let entry_type = extract_entry_type(entry);
            let doi = extract_doi(entry);
            let pages = extract_pages(entry);
            let volume = extract_volume(entry);
            let issue = extract_issue(entry);
            let publisher = extract_publisher(entry);
            let address = extract_location(entry);
            let isbn = extract_isbn(entry);
            let issn = extract_issn(entry);
            let editor = extract_editors(entry, &citation_key);
            let edition = extract_edition(entry);
            let note = extract_note(entry);
            let organization = extract_organization(entry);

            tracing::debug!(
                "Entry {}: processed - title='{}', type={:?}, authors={:?}, year='{}', month='{}'",
                citation_key,
                title,
                entry_type,
                authors,
                pub_year,
                pub_month
            );

            (
                citation_key.clone(),
                BibItem {
                    citation_key,
                    title,
                    authors,
                    pub_month,
                    pub_year,
                    summary,
                    url,
                    index: None,
                    // Extended fields
                    entry_type,
                    doi,
                    pages,
                    volume,
                    issue,
                    publisher,
                    address,
                    isbn,
                    issn,
                    editor,
                    edition,
                    note,
                    organization,
                    // Store original hayagriva Entry for CSL rendering
                    hayagriva_entry: Some(Arc::new(entry.clone())),
                },
            )
        })
        .collect();

    tracing::debug!(
        "Bibliography parsed successfully with {} entries",
        result.len()
    );
    Ok(result)
}

/// Bibliography format type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BibFormat {
    BibTeX,
    Yaml,
}

fn format_string_to_text(fs: &hayagriva::types::FormatString) -> String {
    fs.to_string()
}

fn extract_authors(entry: &hayagriva::Entry, citation_key: &str) -> Vec<Vec<String>> {
    // Try to get authors from the entry
    let persons = entry.authors();

    match persons {
        Some(persons) if !persons.is_empty() => {
            let authors: Vec<Vec<String>> = persons.iter().map(person_to_parts).collect();

            tracing::debug!(
                "Entry {}: extracted {} authors: {:?}",
                citation_key,
                authors.len(),
                authors
            );
            authors
        }
        _ => {
            tracing::warn!("Entry {}: no authors found, using 'N/A'", citation_key);
            vec![vec!["N/A".to_string()]]
        }
    }
}

fn person_to_parts(person: &Person) -> Vec<String> {
    let mut parts = Vec::new();

    // For compatibility with existing format, we'll try to match:
    // [last_name, first_name] or similar structure

    // Add name (family name/last name) - it's a String, not Option
    parts.push(person.name.clone());

    // Add given name (first name)
    if let Some(given) = &person.given_name {
        parts.push(given.to_string());
    }

    // Add prefix if present
    if let Some(prefix) = &person.prefix {
        if parts.len() > 1 {
            parts.insert(0, prefix.to_string());
        } else {
            parts.push(prefix.to_string());
        }
    }

    // Add suffix if present
    if let Some(suffix) = &person.suffix {
        parts.push(suffix.to_string());
    }

    parts
}

fn extract_summary(entry: &hayagriva::Entry, citation_key: &str) -> String {
    match entry.note() {
        Some(note) => {
            let summary = format_string_to_text(note);
            tracing::debug!("Entry {}: found abstract/note", citation_key);
            summary
        }
        None => {
            tracing::debug!(
                "Entry {}: no abstract/note field, using 'N/A'",
                citation_key
            );
            "N/A".to_string()
        }
    }
}

fn extract_url(entry: &hayagriva::Entry, citation_key: &str) -> Option<String> {
    match entry.url() {
        Some(url) => {
            let url_str = url.to_string();
            tracing::debug!("Entry {}: url field = '{}'", citation_key, url_str);
            Some(url_str)
        }
        None => {
            tracing::debug!("Entry {}: no url field", citation_key);
            None
        }
    }
}

fn extract_date(entry: &hayagriva::Entry, citation_key: &str) -> (String, String) {
    let date = entry.date();

    match date {
        Some(date) => {
            let year = date.year.to_string();

            let month = date
                .month
                .map(|m| format!("{:02}", m + 1))
                .unwrap_or_else(|| "N/A".to_string());

            tracing::debug!(
                "Entry {}: extracted date - year='{}', month='{}'",
                citation_key,
                year,
                month
            );

            (year, month)
        }
        None => {
            tracing::debug!(
                "Entry {}: no date field, using 'N/A' for both year and month",
                citation_key
            );
            ("N/A".to_string(), "N/A".to_string())
        }
    }
}

fn extract_entry_type(entry: &hayagriva::Entry) -> Option<String> {
    Some(format!("{:?}", entry.entry_type()))
}

fn extract_doi(entry: &hayagriva::Entry) -> Option<String> {
    entry.doi().map(|d| d.to_string())
}

fn extract_pages(entry: &hayagriva::Entry) -> Option<String> {
    entry.page_range().map(|range| range.to_string())
}

fn extract_volume(entry: &hayagriva::Entry) -> Option<String> {
    entry.volume().map(|v| v.to_string())
}

fn extract_issue(entry: &hayagriva::Entry) -> Option<String> {
    entry.issue().map(|i| i.to_string())
}

fn extract_publisher(entry: &hayagriva::Entry) -> Option<String> {
    entry.publisher().map(|p| {
        // Publisher type doesn't implement Display, so use Debug formatting
        // This will include both name and location if available
        format!("{p:?}")
    })
}

fn extract_location(entry: &hayagriva::Entry) -> Option<String> {
    entry.location().map(format_string_to_text)
}

fn extract_isbn(entry: &hayagriva::Entry) -> Option<String> {
    entry.isbn().map(|isbn| isbn.to_string())
}

fn extract_issn(entry: &hayagriva::Entry) -> Option<String> {
    entry.issn().map(|issn| issn.to_string())
}

fn extract_editors(entry: &hayagriva::Entry, citation_key: &str) -> Option<Vec<Vec<String>>> {
    let editors = entry.editors();

    match editors {
        Some(editors) if !editors.is_empty() => {
            let editor_list: Vec<Vec<String>> = editors.iter().map(person_to_parts).collect();

            tracing::debug!(
                "Entry {}: extracted {} editors: {:?}",
                citation_key,
                editor_list.len(),
                editor_list
            );
            Some(editor_list)
        }
        _ => {
            tracing::debug!("Entry {}: no editors found", citation_key);
            None
        }
    }
}

fn extract_edition(entry: &hayagriva::Entry) -> Option<String> {
    entry.edition().map(|e| e.to_string())
}

fn extract_note(_entry: &hayagriva::Entry) -> Option<String> {
    // Note field was already used for summary/abstract, so we return None here
    // to avoid duplication. The note() method is used in extract_summary().
    None
}

fn extract_organization(entry: &hayagriva::Entry) -> Option<String> {
    entry.organization().map(format_string_to_text)
}
