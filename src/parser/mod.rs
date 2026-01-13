use std::collections::HashMap;

use anyhow::anyhow;
use indexmap::IndexMap;
use mdbook_preprocessor::errors::{Error, Result as MdResult};
use nom_bibtex::Bibtex;
use regex::Regex;

use crate::models::BibItem;

/// Parse bibliography content into a map of BibItems.
pub fn parse_bibliography(raw_content: String) -> MdResult<IndexMap<String, BibItem>, Error> {
    tracing::info!("Parsing bibliography...");

    // Filter quotes (") that may appear in abstracts, etc. and that Bibtex parser doesn't like
    let mut biblatex_content = raw_content.replace('\"', "");
    // Expressions in the content such as R@10 are not parsed well
    let re = Regex::new(r" (?P<before>[A-Za-z])@(?P<after>\d+) ").unwrap();
    biblatex_content = re
        .replace_all(&biblatex_content, " ${before}_at_${after} ")
        .into_owned();

    tracing::info!("Attempting to parse BibTeX content...");
    let bib = match Bibtex::parse(&biblatex_content) {
        Ok(bib) => {
            tracing::info!("Successfully parsed BibTeX content");
            bib
        }
        Err(e) => {
            tracing::error!("Failed to parse BibTeX content: {}", e);
            tracing::error!("This might be due to malformed BibTeX syntax, missing braces, or invalid characters");
            return Err(anyhow!("BibTeX parsing failed: {e}"));
        }
    };

    let biblio = bib.bibliographies();
    tracing::info!("{} bibliography items read", biblio.len());

    let bibliography: IndexMap<String, BibItem> = biblio
        .iter()
        .map(|bib| {
            let citation_key = bib.citation_key().to_string();
            tracing::info!("Processing bibliography entry: {}", citation_key);

            let tm: HashMap<String, String> = bib
                .tags()
                .iter()
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect();

            let authors_str = extract_authors(&tm, &citation_key);
            let title = extract_title(&tm, &citation_key);
            let summary = extract_summary(&tm, &citation_key);
            let url = extract_url(&tm, &citation_key);
            let (pub_year, pub_month) = extract_date(&tm, &citation_key);
            let authors = parse_authors(&authors_str, &citation_key);

            tracing::debug!(
                "Entry {}: final authors list = '{:?}'",
                citation_key,
                authors
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
                },
            )
        })
        .collect();

    tracing::debug!("Bibliography content:\n{:?}", bibliography);
    Ok(bibliography)
}

fn extract_authors(tm: &HashMap<String, String>, citation_key: &str) -> String {
    match tm.get("author") {
        Some(author) => {
            let mut clean_author = author.to_string();
            clean_author.retain(|c| c != '\n');
            tracing::debug!("Entry {}: author field = '{}'", citation_key, clean_author);
            clean_author
        }
        None => {
            tracing::warn!("Entry {}: missing author field, using 'N/A'", citation_key);
            "N/A".to_string()
        }
    }
}

fn extract_title(tm: &HashMap<String, String>, citation_key: &str) -> String {
    match tm.get("title") {
        Some(title_val) => {
            tracing::debug!("Entry {}: title field = '{}'", citation_key, title_val);
            title_val.to_string()
        }
        None => {
            tracing::warn!(
                "Entry {}: missing title field, using 'Not Found'",
                citation_key
            );
            "Not Found".to_string()
        }
    }
}

fn extract_summary(tm: &HashMap<String, String>, citation_key: &str) -> String {
    match tm.get("abstract") {
        Some(abstract_val) => {
            tracing::debug!(
                "Entry {}: abstract field = '{}'",
                citation_key,
                abstract_val
            );
            abstract_val.to_string()
        }
        None => {
            tracing::debug!(
                "Entry {}: missing abstract field, using 'N/A'",
                citation_key
            );
            "N/A".to_string()
        }
    }
}

fn extract_url(tm: &HashMap<String, String>, citation_key: &str) -> Option<String> {
    match tm.get("url") {
        Some(url_val) => match url_val.parse::<String>() {
            Ok(parsed_url) => {
                tracing::debug!("Entry {}: url field = '{}'", citation_key, parsed_url);
                Some(parsed_url)
            }
            Err(e) => {
                tracing::warn!(
                    "Entry {}: failed to parse URL '{}': {}",
                    citation_key,
                    url_val,
                    e
                );
                None
            }
        },
        None => {
            tracing::debug!("Entry {}: missing url field", citation_key);
            None
        }
    }
}

pub fn extract_date(tm: &HashMap<String, String>, citation_key: &str) -> (String, String) {
    if let Some(date_str) = tm.get("date") {
        tracing::debug!(
            "Entry {}: Processing date field: '{}'",
            citation_key,
            date_str
        );
        let mut date = date_str.split('-');
        let year = date.next().unwrap_or("N/A").to_string();
        let month = date
            .next()
            .unwrap_or_else(|| tm.get("month").map(|s| s.as_str()).unwrap_or("N/A"))
            .to_string();
        tracing::debug!(
            "Entry {}: Extracted from date field: year='{}', month='{}'",
            citation_key,
            year,
            month
        );
        (year, month)
    } else {
        tracing::debug!(
            "Entry {}: No date field found, looking for separate year/month fields",
            citation_key
        );
        let year = tm
            .get("year")
            .map(|s| s.as_str())
            .unwrap_or("N/A")
            .to_string();
        let month = tm
            .get("month")
            .map(|s| s.as_str())
            .unwrap_or("N/A")
            .to_string();
        tracing::debug!(
            "Entry {}: Extracted from separate fields: year='{}', month='{}'",
            citation_key,
            year,
            month
        );
        (year, month)
    }
}

fn parse_authors(authors_str: &str, citation_key: &str) -> Vec<Vec<String>> {
    let and_split = Regex::new(r"\band\b").expect("Broken regex");
    let splits = and_split.split(authors_str);
    splits
        .map(|a| {
            let author_parts: Vec<String> =
                a.trim().split(',').map(|b| b.trim().to_string()).collect();
            tracing::debug!("Entry {}: author part = '{:?}'", citation_key, author_parts);
            author_parts
        })
        .collect()
}
