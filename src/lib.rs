#[macro_use]
extern crate lazy_static;

use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::path::PathBuf;

use anyhow::anyhow;
use handlebars::Handlebars;
use log::{debug, error, info, warn};
use mdbook::book::{Book, BookItem, Chapter};
use mdbook::errors::{Error, Result as MdResult};
use mdbook::preprocess::{Preprocessor, PreprocessorContext};
use nom_bibtex::*;
use regex::{CaptureMatches, Captures, Regex};
use serde::{Deserialize, Serialize};

mod file_utils;

static NAME: &str = "bib";
static CSS: &str = include_str!("./render/satancisco.css");
static BIBLIO_HB: &str = include_str!("./render/references.hbs");

pub struct Bibiography;

impl Bibiography {
    pub fn new() -> Bibiography {
        Bibiography
    }
}

/// Bibliography item representation.
/// TODO: Complete with more fields when necessary
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct BibItem {
    /// The citation key.
    pub citation_key: String,
    /// The article's title.
    pub title: String,
    /// The article's author/s.
    pub authors: String,
    /// Pub month.
    pub pub_month: String,
    /// Pub year.
    pub pub_year: String,
    /// Summary/Abstract.
    pub summary: String,
}

impl BibItem {
    /// Create a new bib item with the provided content.
    pub fn new(
        citation_key: &str,
        title: String,
        authors: String,
        pub_month: String,
        pub_year: String,
        summary: String,
    ) -> BibItem {
        BibItem {
            citation_key: citation_key.to_string(),
            title: title,
            authors: authors,
            pub_month: pub_month,
            pub_year: pub_year,
            summary: summary,
        }
    }
}

/// Load bibliography from file. Gets the references and info created from bibliography.
pub(crate) fn load_bibliography<P: AsRef<Path>>(
    biblio_file: P,
) -> MdResult<HashMap<String, BibItem>> {
    log::info!("Loading bibliography from {:?}...", biblio_file.as_ref());

    let biblio_file_ext = file_utils::get_filename_extension(biblio_file.as_ref());
    if biblio_file_ext.unwrap_or_default().to_lowercase() != "bib" {
        warn!(
            "Only bib-based bibliography is supported for now! Yours: {:?}",
            biblio_file.as_ref()
        );
        let out: HashMap<String, BibItem> = HashMap::new();
        return Ok(out);
    }

    let bibtex_content = fs::read_to_string(biblio_file)?.to_string();

    let bibtex = Bibtex::parse(&bibtex_content).unwrap();

    let biblio = bibtex.bibliographies();
    info!("{} bibliography items read", biblio.len());

    let bibliography: HashMap<String, BibItem> = biblio
        .into_iter()
        .map(|bib| {
            let tm: HashMap<String, String> = bib.tags().into_iter().map(|t| t.clone()).collect();
            let mut authors_str = tm.get("author").unwrap().to_string();
            authors_str.retain(|c| c != '\n');
            let authors: Vec<String> = authors_str
                .split("and")
                .map(|a| a.trim().to_string())
                .collect();
            (
                bib.citation_key().to_string(),
                BibItem {
                    citation_key: bib.citation_key().to_string(),
                    title: tm
                        .get("title")
                        .unwrap_or(&"Not Found".to_owned())
                        .to_string(),
                    authors: authors.join(", "),
                    pub_month: tm.get("month").unwrap_or(&"N/A".to_owned()).to_string(),
                    pub_year: tm.get("year").unwrap_or(&"N/A".to_owned()).to_string(),
                    summary: tm.get("abstract").unwrap_or(&"N/A".to_owned()).to_string(),
                },
            )
        })
        .collect();
    debug!("Bibiography content:\n{:?}", bibliography);

    Ok(bibliography)
}

impl Preprocessor for Bibiography {
    fn name(&self) -> &str {
        NAME
    }

    fn run(&self, ctx: &PreprocessorContext, mut book: Book) -> Result<Book, Error> {
        info!("Processor Name: {}", self.name());

        if let Some(cfg) = ctx.config.get_preprocessor(self.name()) {
            // Get references and info the bibliography file specified in the config arg "bibliography"
            // TODO Maybe check the file extension and decide how to process?
            let bibliography = match cfg.get("bibliography") {
                Some(biblio_file_toml) => {
                    let biblio_file = biblio_file_toml.as_str().unwrap();
                    info!("Bibliography file: {}", biblio_file);
                    let biblio_path = ctx.root.join(Path::new(&biblio_file));
                    if !biblio_path.exists() {
                        return Err(anyhow!(format!(
                            "Bibliography file {:?} not found!",
                            biblio_path
                        )));
                    }
                    info!("Bibliography path: {}", biblio_path.display());
                    load_bibliography(biblio_path)
                }
                _ => {
                    warn!("Bibliography file not specified. Skipping processing of bibliography");
                    return Ok(book);
                }
            };

            info!("Bibliography loaded {:?}", bibliography);

            let mut handlebars = Handlebars::new();
            let references_hb = format!("\n\n{}\n\n", BIBLIO_HB);
            handlebars
                .register_template_string("references", references_hb)
                .unwrap();
            debug!("Hanglebars content: {:?}", handlebars);

            let content = match &bibliography {
                Ok(bibliography) => {
                    let mut rendered: String = String::from("");
                    for (_key, value) in &*bibliography {
                        rendered
                            .push_str(handlebars.render("references", &value).unwrap().as_str());
                    }
                    rendered
                }
                Err(e) => {
                    let err_str: String = "Error parsing bibliography".to_owned();
                    error!("{}", err_str);
                    format!("{}: {}", err_str, e.to_string())
                }
            };
            info!("Generated Bib Content: {:?}", content);

            let b = bibliography.unwrap().clone();
            book.for_each_mut(|section: &mut BookItem| {
                if let BookItem::Chapter(ref mut ch) = *section {
                    if let Some(ref chapter_path) = ch.path {
                        info!(
                            "Replacing placeholders({{#cite ..}}) in {}",
                            chapter_path.as_path().display()
                        );
                        let new_content = replace_all_placeholders(&ch.content, &b);
                        ch.content = new_content;
                    }
                }
            });

            info!(
                "Creating new Bibliography chapter with content: {:?}",
                content
            );
            let css_style = format!("<style>{}</style>\n\n", CSS); // Add the style css for the biblio
            let biblio_content = format!("{}{}", css_style, content);
            let bib_chapter = Chapter::new(
                "Bibliography",
                format!("# Bibliography\n{}", biblio_content),
                PathBuf::from("bibliography.md"),
                Vec::new(),
            );
            book.push_item(bib_chapter);
        }
        Ok(book)
    }

    fn supports_renderer(&self, renderer: &str) -> bool {
        renderer != "not-supported"
    }
}

fn replace_all_placeholders(s: &str, bibliography: &HashMap<String, BibItem>) -> String {
    // When replacing one thing in a string by something with a different length,
    // the indices after that will not correspond,
    // we therefore have to store the difference to correct this
    let mut previous_end_index = 0;
    let mut replaced = String::new();

    for placeholder in find_placeholders(s) {
        replaced.push_str(&s[previous_end_index..placeholder.start_index]);

        match placeholder.render_with_path(bibliography) {
            Ok(new_content) => {
                replaced.push_str(&new_content);
                previous_end_index = placeholder.end_index;
            }
            Err(e) => {
                error!("Error updating \"{}\", {}", placeholder.placeholder_text, e);
                for cause in e.chain().skip(1) {
                    warn!("Caused By: {}", cause);
                }

                // This should make sure we include the raw `{{# ... }}` snippet
                // in the page content if there are any errors.
                previous_end_index = placeholder.start_index;
            }
        }
    }

    replaced.push_str(&s[previous_end_index..]);
    replaced
}

fn parse_cite(cite: &str) -> PlaceholderType {
    PlaceholderType::Cite(cite.to_owned())
}

#[derive(PartialEq, Debug, Clone)]
enum PlaceholderType {
    Cite(String),
}

#[derive(PartialEq, Debug, Clone)]
struct Placeholder<'a> {
    start_index: usize,
    end_index: usize,
    placeholder_type: PlaceholderType,
    placeholder_text: &'a str,
}

impl<'a> Placeholder<'a> {
    fn from_capture(cap: Captures<'a>) -> Option<Placeholder<'a>> {
        let placeholder_type = match (cap.get(0), cap.get(1), cap.get(2)) {
            (_, Some(typ), Some(rest)) => {
                let mut path_props = rest.as_str().split_whitespace();
                let file_arg = path_props.next();

                match (typ.as_str(), file_arg) {
                    ("cite", Some(cite)) => Some(parse_cite(cite)),
                    _ => None,
                }
            }
            _ => None,
        };

        placeholder_type.and_then(|plh_type| {
            cap.get(0).map(|mat| Placeholder {
                start_index: mat.start(),
                end_index: mat.end(),
                placeholder_type: plh_type,
                placeholder_text: mat.as_str(),
            })
        })
    }

    fn render_with_path(&self, bibliography: &HashMap<String, BibItem>) -> MdResult<String> {
        match self.placeholder_type {
            PlaceholderType::Cite(ref cite) => {
                if bibliography.contains_key(cite) {
                    Ok(format!("\\[[{}](bibliography.html#{})\\]", cite, cite))
                } else {
                    Ok(format!("\\[Unknown bib ref: {}\\]", cite))
                }
            }
        }
    }
}

struct PlaceholderIter<'a>(CaptureMatches<'a, 'a>);

impl<'a> Iterator for PlaceholderIter<'a> {
    type Item = Placeholder<'a>;
    fn next(&mut self) -> Option<Placeholder<'a>> {
        for cap in &mut self.0 {
            if let Some(plh) = Placeholder::from_capture(cap) {
                debug!("Placeholder found: {:?}", plh);
                return Some(plh);
            }
        }
        None
    }
}

fn find_placeholders(contents: &str) -> PlaceholderIter<'_> {
    // lazily compute following regex
    // r"\\\{\{#.*\}\}|\{\{#([a-zA-Z0-9]+)\s*([a-zA-Z0-9_.\-:/\\\s]+)\}\}")?;
    lazy_static! {
        static ref RE: Regex = Regex::new(
            r"(?x)                       # insignificant whitespace mode
            \\\{\{\#.*\}\}               # match escaped placeholder
            |                            # or
            \{\{\s*                      # placeholder opening parens and whitespace
            \#([a-zA-Z0-9_]+)            # placeholder type
            \s+                          # separating whitespace
            ([a-zA-Z0-9\s_.\-:/\\\+]+)   # placeholder target path and space separated properties
            \s*\}\}                      # whitespace and placeholder closing parens"
        )
        .unwrap();
    }
    PlaceholderIter(RE.captures_iter(contents))
}

#[cfg(test)]
mod tests;
