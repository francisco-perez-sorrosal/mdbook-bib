#[macro_use]
extern crate lazy_static;

use std::collections::{HashMap, HashSet};
use std::fs;
use std::io::Read;
use std::path::Path;
use std::path::PathBuf;

use crate::config::Config;
use anyhow::anyhow;
use handlebars::Handlebars;
use log::{debug, info, warn};
use mdbook::book::{Book, BookItem, Chapter};
use mdbook::errors::{Error, Result as MdResult};
use mdbook::preprocess::{Preprocessor, PreprocessorContext};
use nom_bibtex::*;
use regex::{CaptureMatches, Captures, Regex};
use reqwest::blocking::Response;
use serde::{Deserialize, Serialize};

mod config;
mod file_utils;

static NAME: &str = "bib";

pub struct Bibiography;

impl Default for Bibiography {
    fn default() -> Bibiography {
        Bibiography
    }
}

impl Bibiography {
    // Get references and info from the bibliography file specified in the config
    // arg "bibliography" or in Zotero
    fn retrieve_bibliography_content(
        ctx: &PreprocessorContext,
        cfg: &Config,
    ) -> Result<String, Error> {
        let bib_content = match &cfg.bibliography {
            Some(biblio_file) => {
                info!("Bibliography file: {}", biblio_file);
                let mut biblio_path = ctx.root.join(ctx.config.book.src.to_owned());
                biblio_path = biblio_path.join(Path::new(&biblio_file));
                if !biblio_path.exists() {
                    Err(anyhow!(format!(
                        "Bibliography file {:?} not found!",
                        biblio_path
                    )))
                } else {
                    info!("Bibliography path: {}", biblio_path.display());
                    load_bibliography(biblio_path)
                }
            }
            _ => {
                warn!("Bibliography file not specified. Trying download from Zotero");
                match &cfg.zotero_uid {
                    Some(uid) => {
                        let user_id = uid.to_string();
                        let bib_str = download_bib_from_zotero(user_id).unwrap_or_default();
                        if !bib_str.is_empty() {
                            let biblio_path = ctx.root.join(Path::new("my_zotero.bib"));
                            info!("Saving Zotero bibliography to {:?}", biblio_path);
                            let _ = fs::write(biblio_path, bib_str.to_owned());
                            Ok(bib_str)
                        } else {
                            // warn!("Bib content retrieved from Zotero is empty!");
                            Err(anyhow!("Bib content retrieved from Zotero is empty!"))
                        }
                    }
                    _ => Err(anyhow!("Zotero user id not specified either :(")),
                }
            }
        };
        bib_content
    }

    fn generate_bibliography_html(
        bibliography: &HashMap<String, BibItem>,
        cited: &HashSet<String>,
        cited_only: bool,
        references_tpl: String,
    ) -> String {
        let mut handlebars = Handlebars::new();
        handlebars
            .register_template_string("references", references_tpl)
            .unwrap();
        debug!("Handlebars content: {:?}", handlebars);

        let mut content: String = String::from("");
        for (key, value) in bibliography {
            if !cited_only || cited.contains(key) {
                content.push_str(handlebars.render("references", &value).unwrap().as_str());
            }
        }

        debug!("Generated Bib Content: {:?}", content);
        content
    }

    fn expand_cite_references_in_book(
        book: &mut Book,
        bib_as_html_filepath: PathBuf,
        bibliography: &HashMap<String, BibItem>,
    ) -> HashSet<String> {
        let mut cited = HashSet::new();
        book.for_each_mut(|section: &mut BookItem| {
            if let BookItem::Chapter(ref mut ch) = *section {
                if let Some(ref chapter_path) = ch.path {
                    info!(
                        "Replacing placeholders: {{#cite ...}} and @@citation in {}",
                        chapter_path.as_path().display()
                    );
                    let new_content = replace_all_placeholders(
                        &ch.content,
                        bib_as_html_filepath.to_owned(),
                        bibliography,
                        &mut cited,
                    );
                    ch.content = new_content;
                }
            }
        });
        cited
    }

    fn create_bibliography_chapter(
        title: String,
        js_html_part: String,
        css_html_part: String,
        biblio_html_part: String,
    ) -> Chapter {
        let html_content = format!("{}\n{}\n{}", js_html_part, css_html_part, biblio_html_part);
        debug!(
            "Creating new Bibliography chapter (with title: \"{}\") with content: {:?}",
            title, html_content
        );

        Chapter::new(
            &title,
            format!("# {}\n{}", title, html_content),
            PathBuf::from("bibliography.md"),
            Vec::new(),
        )
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
    /// The article's url.
    pub url: Option<String>,
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
        url: Option<String>,
    ) -> BibItem {
        BibItem {
            citation_key: citation_key.to_string(),
            title,
            authors,
            pub_month,
            pub_year,
            summary,
            url,
        }
    }
}

/// Load bibliography from file
pub(crate) fn load_bibliography<P: AsRef<Path>>(biblio_file: P) -> MdResult<String> {
    log::info!("Loading bibliography from {:?}...", biblio_file.as_ref());

    let biblio_file_ext = file_utils::get_filename_extension(biblio_file.as_ref());
    if biblio_file_ext.unwrap_or_default().to_lowercase() != "bib" {
        warn!(
            "Only biblatex-based bibliography is supported for now! Yours: {:?}",
            biblio_file.as_ref()
        );
        return Ok("".to_string());
    }
    Ok(fs::read_to_string(biblio_file)?)
}

fn extract_biblio_data_and_link_info(res: &mut Response) -> (String, String) {
    let mut biblio_chunk = String::new();
    let _ = res.read_to_string(&mut biblio_chunk);
    let link_info_in_header = res.headers().get("link");
    debug!("Header Link content: {:?}", link_info_in_header);
    let link_info_as_str = link_info_in_header.unwrap().to_str();

    (link_info_as_str.unwrap().to_string(), biblio_chunk)
}

/// Download bibliography from Zotero
pub(crate) fn download_bib_from_zotero(user_id: String) -> MdResult<String, Error> {
    let mut url = format!("https://api.zotero.org/users/{}/items?format=biblatex&style=biblatex&limit=100&sort=creator&v=3", user_id);
    info!("Zotero's URL biblio source:\n{:?}", url);
    let mut res = reqwest::blocking::get(&url)?;
    if res.status().is_client_error() || res.status().is_client_error() {
        Err(anyhow!(format!(
            "Error accessing Zotero API {:?}",
            res.error_for_status()
        )))
    } else {
        let (mut link_str, mut bib_content) = extract_biblio_data_and_link_info(&mut res);
        while link_str.contains("next") {
            // Extract next chunk URL
            let next_idx = link_str.find("rel=\"next\"").unwrap();
            let end_bytes = next_idx - 3; // The > of the "next" link is 3 chars before rel=\"next\" pattern
            let slice = &link_str[..end_bytes];
            let start_bytes = slice.rfind('<').unwrap_or(0);
            url = link_str[(start_bytes + 1)..end_bytes].to_string();
            info!("Next biblio chunk URL:\n{:?}", url);
            res = reqwest::blocking::get(&url)?;
            let (new_link_str, new_bib_part) = extract_biblio_data_and_link_info(&mut res);
            link_str = new_link_str;
            bib_content.push_str(&new_bib_part);
        }
        Ok(bib_content)
    }
}

/// Gets the references and info created from bibliography.
pub(crate) fn build_bibliography(raw_content: String) -> MdResult<HashMap<String, BibItem>, Error> {
    log::info!("Building bibliography...");

    // Filter quotes (") that may appear in abstracts, etc. and that Bibtex parser doesn't like
    let mut biblatex_content = raw_content.replace("\"", "");
    // Expressions in the content such as R@10 are not parsed well
    let re = Regex::new(r" (?P<before>[A-Za-z])@(?P<after>\d+) ").unwrap();
    biblatex_content = re
        .replace_all(&biblatex_content, " ${before}_at_${after} ")
        .into_owned();
    let bib = Bibtex::parse(&biblatex_content)?;

    let biblio = bib.bibliographies();
    info!("{} bibliography items read", biblio.len());

    let bibliography: HashMap<String, BibItem> = biblio
        .iter()
        .map(|bib| {
            let tm: HashMap<String, String> = bib.tags().iter().cloned().collect();
            let mut authors_str = tm.get("author").unwrap_or(&"N/A".to_owned()).to_string();
            authors_str.retain(|c| c != '\n');

            info!("{:?}", &tm);
            let (pub_year, pub_month) = extract_date(&tm);

            let authors: Vec<String> = authors_str
                .split("and")
                .map(|a| a.trim().to_string())
                .collect();
            let url: Option<String> = tm.get("url").map(|u| (*u.to_owned()).parse().unwrap());

            (
                bib.citation_key().to_string(),
                BibItem {
                    citation_key: bib.citation_key().to_string(),
                    title: tm
                        .get("title")
                        .unwrap_or(&"Not Found".to_owned())
                        .to_string(),
                    authors: authors.join(", "),
                    pub_month,
                    pub_year,
                    summary: tm.get("abstract").unwrap_or(&"N/A".to_owned()).to_string(),
                    url,
                },
            )
        })
        .collect();
    debug!("Bibiography content:\n{:?}", bibliography);

    Ok(bibliography)
}

fn extract_date(tm: &HashMap<String, String>) -> (String, String) {
    if let Some(date_str) = tm.get("date") {
        let mut date = date_str.split('-');
        let year = date.next().unwrap_or("N/A").to_string();
        let month = date
            .next()
            .unwrap_or_else(|| tm.get("month").map(|s| s.as_str()).unwrap_or("N/A"))
            .to_string();
        (year, month)
    } else {
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
        (year, month)
    }
}

impl Preprocessor for Bibiography {
    fn name(&self) -> &str {
        NAME
    }

    fn run(&self, ctx: &PreprocessorContext, mut book: Book) -> Result<Book, anyhow::Error> {
        info!("Processor Name: {}", self.name());
        let book_src_root = ctx.root.join(ctx.config.book.src.to_owned());
        let table = ctx.config.get_preprocessor(self.name());
        let config = match Config::build_from(table, book_src_root) {
            Ok(config) => config,
            Err(err) => {
                warn!(
                    "Error reading configuration. Skipping processing: {:?}",
                    err
                );
                return Ok(book);
            }
        };

        let bib_content = Bibiography::retrieve_bibliography_content(ctx, &config);

        if bib_content.is_err() {
            warn!(
                "Raw Bibliography content couldn't be retrieved. Skipping processing: {:?}",
                bib_content.err()
            );
            return Ok(book);
        }

        let bibliography = build_bibliography(bib_content?);
        if bibliography.is_err() {
            warn!(
                "Error building Bibliography from raw content. Skipping render: {:?}",
                bibliography.err()
            );
            return Ok(book);
        }

        let bib = bibliography.unwrap();

        let mut bib_as_html_filepath = ctx.root.join(ctx.config.build.build_dir.to_owned());
        bib_as_html_filepath = bib_as_html_filepath.join("bibliography.html");
        debug!("Path to html bib file: {:?}", bib_as_html_filepath);
        let cited =
            Bibiography::expand_cite_references_in_book(&mut book, bib_as_html_filepath, &bib);

        let bib_content_html = Bibiography::generate_bibliography_html(
            &bib,
            &cited,
            config.cited_only,
            config.bib_hb_html,
        );

        let bib_chapter = Bibiography::create_bibliography_chapter(
            config.title,
            config.js_html,
            config.css_html,
            bib_content_html,
        );

        book.push_item(bib_chapter);

        Ok(book)
    }

    fn supports_renderer(&self, renderer: &str) -> bool {
        renderer != "not-supported"
    }
}

fn replace_all_placeholders<'a>(
    s: &'a str,
    bib_as_html_filepath: PathBuf,
    bibliography: &HashMap<String, BibItem>,
    cited: &mut HashSet<String>,
) -> String {
    // When replacing one thing in a string by something with a different length,
    // the indices after that will not correspond,
    // we therefore have to store the difference to correct this
    let mut previous_end_index = 0;
    let mut replaced = String::new();

    for placeholder in find_placeholders(s) {
        replaced.push_str(&s[previous_end_index..placeholder.start_index]);
        replaced
            .push_str(&placeholder.render_with_path(bib_as_html_filepath.to_owned(), bibliography));
        previous_end_index = placeholder.end_index;

        match placeholder.placeholder_type {
            PlaceholderType::Cite(ref cite) | PlaceholderType::AtCite(ref cite) => {
                cited.insert(cite.to_owned());
            }
        }
    }
    // TODO Maybe look how to combine two iterators to avoid the duplicated code below
    for placeholder in find_at_placeholders(s) {
        replaced.push_str(&s[previous_end_index..placeholder.start_index]);
        replaced
            .push_str(&placeholder.render_with_path(bib_as_html_filepath.to_owned(), bibliography));
        previous_end_index = placeholder.end_index;

        match placeholder.placeholder_type {
            PlaceholderType::Cite(ref cite) | PlaceholderType::AtCite(ref cite) => {
                cited.insert(cite.to_owned());
            }
        }
    }

    replaced.push_str(&s[previous_end_index..]);
    replaced
}

fn parse_cite(cite: &str) -> PlaceholderType {
    PlaceholderType::Cite(cite.to_owned())
}

fn parse_at_cite(cite: &str) -> PlaceholderType {
    PlaceholderType::AtCite(cite.to_owned())
}

#[derive(PartialEq, Debug, Clone)]
enum PlaceholderType {
    Cite(String),
    AtCite(String),
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
                    ("@@", Some(cite)) => Some(parse_at_cite(cite)),
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

    fn render_with_path(
        &self,
        bib_as_html_filepath: PathBuf,
        bibliography: &HashMap<String, BibItem>,
    ) -> String {
        match self.placeholder_type {
            PlaceholderType::Cite(ref cite) | PlaceholderType::AtCite(ref cite) => {
                if bibliography.contains_key(cite) {
                    format!(
                        "\\[[{}]({}#{})\\]",
                        cite,
                        bib_as_html_filepath.into_os_string().into_string().unwrap(),
                        cite
                    )
                } else {
                    format!("\\[Unknown bib ref: {}\\]", cite)
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

const AT_REF_PATTERN: &str = r##"(@@)([^\[\]\s\.,;"#'()={}%]+)"##;
fn find_at_placeholders(contents: &str) -> PlaceholderIter<'_> {
    lazy_static! {
        static ref REF_REGEX: Regex = Regex::new(AT_REF_PATTERN).unwrap();
    }
    PlaceholderIter(REF_REGEX.captures_iter(contents))
}

#[cfg(test)]
mod tests;
