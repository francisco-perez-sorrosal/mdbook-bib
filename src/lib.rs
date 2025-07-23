#[macro_use]
extern crate lazy_static;

use std::collections::{HashMap, HashSet};
use std::fs;
use std::io::Read;
use std::path::{Component, Path, PathBuf};

use crate::config::{Config, SortOrder};
use anyhow::anyhow;
use handlebars::Handlebars;
use indexmap::IndexMap;
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
static BIB_OUT_FILE: &str = "bibliography";

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
                let mut biblio_path = ctx.root.join(&ctx.config.book.src);
                biblio_path = biblio_path.join(Path::new(&biblio_file));
                if !biblio_path.exists() {
                    Err(anyhow!(format!(
                        "Bibliography file {biblio_path:?} not found!",
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
                            let _ = fs::write(biblio_path, &bib_str);
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
        bibliography: &IndexMap<String, BibItem>,
        cited: &HashSet<String>,
        cited_only: bool,
        references_tpl: String,
        order: SortOrder,
    ) -> String {
        let mut handlebars = Handlebars::new();
        handlebars
            .register_template_string("references", references_tpl)
            .unwrap();
        debug!("Handlebars content: {:?}", handlebars);

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

        let mut content: String = String::from("");
        for (key, value) in sorted {
            if !cited_only || cited.contains(key) {
                content.push_str(handlebars.render("references", &value).unwrap().as_str());
            }
        }

        debug!("Generated Bib Content: {:?}", content);
        content
    }

    fn expand_cite_references_in_book(
        book: &mut Book,
        bibliography: &mut IndexMap<String, BibItem>,
        citation_tpl: &str,
    ) -> HashSet<String> {
        let mut cited = HashSet::new();
        let mut last_index = 0;
        book.for_each_mut(|section: &mut BookItem| {
            if let BookItem::Chapter(ref mut ch) = *section {
                if let Some(ref chapter_path) = ch.path {
                    info!(
                        "Replacing placeholders: {{#cite ...}} and @@citation in {}",
                        chapter_path.as_path().display()
                    );
                    let new_content = replace_all_placeholders(
                        ch,
                        bibliography,
                        &mut cited,
                        citation_tpl,
                        &mut last_index,
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
        let html_content = format!("{js_html_part}\n{css_html_part}\n{biblio_html_part}");
        debug!(
            "Creating new Bibliography chapter (with title: \"{}\") with content: {:?}",
            title, html_content
        );

        Chapter::new(
            &title,
            format!("# {title}\n{html_content}"),
            PathBuf::from(format!("{BIB_OUT_FILE}.md")),
            Vec::new(),
        )
    }
}

/// Bibliography item representation.
/// TODO: Complete with more fields when necessary
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct BibItem {
    /// The citation key.
    pub citation_key: String,
    /// The article's title.
    pub title: String,
    /// The article's author/s.
    pub authors: Vec<Vec<String>>,
    /// Pub month.
    pub pub_month: String,
    /// Pub year.
    pub pub_year: String,
    /// Summary/Abstract.
    pub summary: String,
    /// The article's url.
    pub url: Option<String>,
    /// The item's index for first citation in the book.
    pub index: Option<u32>,
}

impl BibItem {
    /// Create a new bib item with the provided content.
    pub fn new(
        citation_key: &str,
        title: String,
        authors: Vec<Vec<String>>,
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
            index: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct Citation {
    pub item: BibItem,
    pub path: String,
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
    let mut url = format!("https://api.zotero.org/users/{user_id}/items?format=biblatex&style=biblatex&limit=100&sort=creator&v=3");
    info!("Zotero's URL biblio source:\n{url:?}");
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
pub(crate) fn build_bibliography(
    raw_content: String,
) -> MdResult<IndexMap<String, BibItem>, Error> {
    log::info!("Building bibliography...");

    // Filter quotes (") that may appear in abstracts, etc. and that Bibtex parser doesn't like
    let mut biblatex_content = raw_content.replace('\"', "");
    // Expressions in the content such as R@10 are not parsed well
    let re = Regex::new(r" (?P<before>[A-Za-z])@(?P<after>\d+) ").unwrap();
    biblatex_content = re
        .replace_all(&biblatex_content, " ${before}_at_${after} ")
        .into_owned();
    let bib = Bibtex::parse(&biblatex_content)?;

    let biblio = bib.bibliographies();
    info!("{} bibliography items read", biblio.len());

    let bibliography: IndexMap<String, BibItem> = biblio
        .iter()
        .map(|bib| {
            let tm: HashMap<String, String> = bib
                .tags()
                .iter()
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect();
            let mut authors_str = tm.get("author").unwrap_or(&"N/A".to_owned()).to_string();
            authors_str.retain(|c| c != '\n');

            info!("{:?}", &tm);
            let (pub_year, pub_month) = extract_date(&tm);
            let and_split = Regex::new(r"\band\b").expect("Broken regex");
            let splits = and_split.split(&authors_str);
            let authors: Vec<Vec<String>> = splits
                .map(|a| a.trim().split(',').map(|b| b.trim().to_string()).collect())
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
                    authors,
                    pub_month,
                    pub_year,
                    summary: tm.get("abstract").unwrap_or(&"N/A".to_owned()).to_string(),
                    url,
                    index: None,
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
        let book_src_root = ctx.root.join(&ctx.config.book.src);
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

        let mut bib = bibliography.unwrap();
        let cited =
            Bibiography::expand_cite_references_in_book(&mut book, &mut bib, &config.cite_hb_html);

        let bib_content_html = Bibiography::generate_bibliography_html(
            &bib,
            &cited,
            config.cited_only,
            config.bib_hb_html,
            config.order,
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

fn replace_all_placeholders(
    chapter: &Chapter,
    bibliography: &mut IndexMap<String, BibItem>,
    cited: &mut HashSet<String>,
    citation_tpl: &str,
    last_index: &mut u32,
) -> String {
    let mut handlebars = Handlebars::new();
    handlebars
        .register_template_string("citation", citation_tpl)
        .unwrap();
    debug!("Handlebars content: {:?}", handlebars);
    // When replacing one thing in a string by something with a different length,
    // the indices after that will not correspond,
    // we therefore have to store the difference to correct this
    let mut previous_end_index = 0;
    let mut replaced = String::new();

    let chapter_path = chapter
        .path
        .as_deref()
        .unwrap_or_else(|| std::path::Path::new(""));

    for placeholder in find_placeholders(&chapter.content) {
        replaced.push_str(&chapter.content[previous_end_index..placeholder.start_index]);
        replaced.push_str(&placeholder.render_with_path(
            chapter_path,
            bibliography,
            &handlebars,
            last_index,
        ));
        previous_end_index = placeholder.end_index;

        match placeholder.placeholder_type {
            PlaceholderType::Cite(ref cite) | PlaceholderType::AtCite(ref cite) => {
                cited.insert(cite.to_owned());
            }
        }
    }

    replaced.push_str(&chapter.content[previous_end_index..]);
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
        source_file: &std::path::Path,
        bibliography: &mut IndexMap<String, BibItem>,
        handlebars: &Handlebars,
        last_index: &mut u32,
    ) -> String {
        match self.placeholder_type {
            PlaceholderType::Cite(ref cite) | PlaceholderType::AtCite(ref cite) => {
                if bibliography.contains_key(cite) {
                    let path_to_root = breadcrumbs_up_to_root(source_file);
                    let item = bibliography.get_mut(cite).unwrap();
                    if item.index.is_none() {
                        *last_index += 1;
                        item.index = Some(*last_index);
                    }
                    let citation = Citation {
                        item: item.to_owned(),
                        path: format!("{path_to_root}{BIB_OUT_FILE}.html"),
                    };
                    handlebars
                        .render("citation", &citation)
                        .unwrap()
                        .as_str()
                        .to_string()
                } else {
                    format!("\\[Unknown bib ref: {cite}\\]")
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

// lazily compute following regex
// r"\\\{\{#.*\}\}|\{\{#([a-zA-Z0-9]+)\s*([a-zA-Z0-9_.\-:/\\\s]+)\}\}")?;
const REF_PATTERN: &str = r"
(?x)                       # insignificant whitespace mode
\\\{\{\#.*\}\}               # match escaped placeholder
|                            # or
\{\{\s*                      # placeholder opening parens and whitespace
\#([a-zA-Z0-9_]+)            # placeholder type
\s+                          # separating whitespace
([a-zA-Z0-9\s_.\-:/\\\+]+)   # placeholder target path and space separated properties
\s*\}\}                      # whitespace and placeholder closing parens";
const AT_REF_PATTERN: &str = r##"(@@)([^\[\]\s\.,;"#'()={}%]+)"##;
fn find_placeholders(contents: &str) -> Vec<Placeholder> {
    lazy_static! {
        static ref REF_REGEX: Regex = Regex::new(REF_PATTERN).unwrap(); // Cite placeholders of type {{ cite }}
        static ref AT_REF_REGEX: Regex = Regex::new(AT_REF_PATTERN).unwrap(); // Cite placeholders of type @@cite
    }
    PlaceholderIter(REF_REGEX.captures_iter(contents))
        .chain(PlaceholderIter(AT_REF_REGEX.captures_iter(contents)))
        .collect()
}

fn breadcrumbs_up_to_root(source_file: &std::path::Path) -> String {
    if source_file.as_os_str().is_empty() {
        return "".into();
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

#[cfg(test)]
mod tests;
