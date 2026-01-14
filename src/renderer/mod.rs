use std::collections::HashSet;

use indexmap::IndexMap;

use crate::backend::BibliographyBackend;
use crate::config::SortOrder;
use crate::models::BibItem;

/// Generate bibliography HTML from BibItems using the specified backend.
pub fn generate_bibliography_html(
    bibliography: &IndexMap<String, BibItem>,
    cited: &HashSet<String>,
    cited_only: bool,
    backend: &dyn BibliographyBackend,
    order: SortOrder,
) -> String {
    let sorted: Vec<(&str, &BibItem)> = match order {
        SortOrder::None => bibliography.iter().map(|(k, v)| (k.as_str(), v)).collect(),
        SortOrder::Key => {
            let mut v: Vec<(&str, &BibItem)> =
                bibliography.iter().map(|(k, v)| (k.as_str(), v)).collect();
            v.sort_by_key(|item| item.0);
            v
        }
        SortOrder::Author => {
            let empty = "!".to_string();
            let mut v: Vec<(&str, &BibItem)> =
                bibliography.iter().map(|(k, v)| (k.as_str(), v)).collect();
            v.sort_by_cached_key(|item| {
                let val: &str = item
                    .1
                    .authors
                    .first()
                    .map(|vec| vec.first().unwrap_or(&empty))
                    .unwrap_or(&empty);
                val
            });
            v
        }
        SortOrder::Index => {
            let mut v: Vec<(&str, &BibItem)> =
                bibliography.iter().map(|(k, v)| (k.as_str(), v)).collect();
            v.sort_by_key(|item| item.1.index);
            v
        }
    };

    let mut content = String::new();
    for (key, value) in sorted {
        if !cited_only || cited.contains(key) {
            match backend.format_reference(value) {
                Ok(html) => content.push_str(&html),
                Err(e) => {
                    tracing::error!("Failed to format reference for '{}': {}", key, e);
                    content.push_str(&format!(
                        "<div class='error'>Error formatting reference: {key}</div>"
                    ));
                }
            }
        }
    }

    tracing::debug!("Generated Bib Content: {:?}", content);
    content
}
