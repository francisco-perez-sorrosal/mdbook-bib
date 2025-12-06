#[macro_use]
extern crate lazy_static;

use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::io::Read;
use std::path::{Component, Path, PathBuf};

use crate::config::{Config, SortOrder};
use anyhow::anyhow;
use handlebars::Handlebars;
use indexmap::IndexMap;
use log::{debug, error, info, warn};
use mdbook_preprocessor::book::{Book, BookItem, Chapter};
use mdbook_preprocessor::errors::{Error, Result as MdResult};
use mdbook_preprocessor::{Preprocessor, PreprocessorContext};
use nom_bibtex::*;
use regex::Regex;
use reqwest::blocking::Response;
use serde::{Deserialize, Serialize};

mod config;
mod file_utils;

static NAME: &str = "bib";
static BIB_OUT_FILE: &str = "bibliography";

pub struct Bibliography;

impl Default for Bibliography {
    fn default() -> Bibliography {
        Bibliography
    }
}

impl Bibliography {
    // Get references and info from the bibliography file specified in the config
    // arg "bibliography" or in Zotero
    fn retrieve_bibliography_content(
        ctx: &PreprocessorContext,
        cfg: &Config,
    ) -> Result<String, Error> {
        let bib_content = match &cfg.bibliography {
            Some(biblio_file) => {
                info!("Bibliography file: {biblio_file}");
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
                            info!("Saving Zotero bibliography to {biblio_path:?}");
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
        debug!("Handlebars content: {handlebars:?}");

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

        debug!("Generated Bib Content: {content:?}");
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
            "Creating new Bibliography chapter (with title: \"{title}\") with content: {html_content:?}"
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
    debug!("Header Link content: {link_info_in_header:?}");
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
            info!("Next biblio chunk URL:\n{url:?}");
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

    info!("Attempting to parse BibTeX content...");
    let bib = match Bibtex::parse(&biblatex_content) {
        Ok(bib) => {
            info!("Successfully parsed BibTeX content");
            bib
        }
        Err(e) => {
            error!("Failed to parse BibTeX content: {e}");
            error!("This might be due to malformed BibTeX syntax, missing braces, or invalid characters");
            return Err(anyhow!("BibTeX parsing failed: {e}"));
        }
    };

    let biblio = bib.bibliographies();
    info!("{} bibliography items read", biblio.len());

    let bibliography: IndexMap<String, BibItem> = biblio
        .iter()
        .map(|bib| {
            let citation_key = bib.citation_key().to_string();
            info!("Processing bibliography entry: {citation_key}");

            let tm: HashMap<String, String> = bib
                .tags()
                .iter()
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect();

            // Process authors with explicit error handling
            let authors_str = match tm.get("author") {
                Some(author) => {
                    let mut clean_author = author.to_string();
                    clean_author.retain(|c| c != '\n');
                    debug!("Entry {citation_key}: author field = '{clean_author}'");
                    clean_author
                }
                None => {
                    warn!("Entry {citation_key}: missing author field, using 'N/A'");
                    "N/A".to_string()
                }
            };

            // Process title with explicit error handling
            let title = match tm.get("title") {
                Some(title_val) => {
                    debug!("Entry {citation_key}: title field = '{title_val}'");
                    title_val.to_string()
                }
                None => {
                    warn!("Entry {citation_key}: missing title field, using 'Not Found'");
                    "Not Found".to_string()
                }
            };

            // Process abstract/summary with explicit error handling
            let summary = match tm.get("abstract") {
                Some(abstract_val) => {
                    debug!("Entry {citation_key}: abstract field = '{abstract_val}'");
                    abstract_val.to_string()
                }
                None => {
                    debug!("Entry {citation_key}: missing abstract field, using 'N/A'");
                    "N/A".to_string()
                }
            };

            // Process URL with explicit error handling
            let url: Option<String> = match tm.get("url") {
                Some(url_val) => match url_val.parse::<String>() {
                    Ok(parsed_url) => {
                        debug!("Entry {citation_key}: url field = '{parsed_url}'");
                        Some(parsed_url)
                    }
                    Err(e) => {
                        warn!("Entry {citation_key}: failed to parse URL '{url_val}': {e}");
                        None
                    }
                },
                None => {
                    debug!("Entry {citation_key}: missing url field");
                    None
                }
            };

            // Process date with explicit error handling
            let (pub_year, pub_month) = extract_date(&tm);
            debug!("Entry {citation_key}: date fields = year='{pub_year}', month='{pub_month}'");

            // Process authors list with explicit error handling
            let and_split = Regex::new(r"\band\b").expect("Broken regex");
            let splits = and_split.split(&authors_str);
            let authors: Vec<Vec<String>> = splits
                .map(|a| {
                    let author_parts: Vec<String> =
                        a.trim().split(',').map(|b| b.trim().to_string()).collect();
                    debug!("Entry {citation_key}: author part = '{author_parts:?}'");
                    author_parts
                })
                .collect();

            debug!("Entry {citation_key}: final authors list = '{authors:?}'");

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
    debug!("Bibiography content:\n{bibliography:?}");

    Ok(bibliography)
}

fn extract_date(tm: &HashMap<String, String>) -> (String, String) {
    if let Some(date_str) = tm.get("date") {
        debug!("Processing date field: '{date_str}'");
        let mut date = date_str.split('-');
        let year = date.next().unwrap_or("N/A").to_string();
        let month = date
            .next()
            .unwrap_or_else(|| tm.get("month").map(|s| s.as_str()).unwrap_or("N/A"))
            .to_string();
        debug!("Extracted from date field: year='{year}', month='{month}'");
        (year, month)
    } else {
        debug!("No date field found, looking for separate year/month fields");
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
        debug!("Extracted from separate fields: year='{year}', month='{month}'");
        (year, month)
    }
}

impl Preprocessor for Bibliography {
    fn name(&self) -> &str {
        NAME
    }

    fn run(&self, ctx: &PreprocessorContext, mut book: Book) -> Result<Book, anyhow::Error> {
        info!("Processor Name: {}", self.name());
        let book_src_root = ctx.root.join(&ctx.config.book.src);
        let table = ctx
            .config
            .get::<toml::value::Table>("preprocessor.bib")
            .unwrap();
        let config = match Config::build_from(table.as_ref(), book_src_root) {
            Ok(config) => config,
            Err(err) => {
                warn!("Error reading configuration. Skipping processing: {err:?}");
                return Ok(book);
            }
        };

        let bib_content = Bibliography::retrieve_bibliography_content(ctx, &config);

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

        if config.add_bib_in_each_chapter {
            add_bib_at_end_of_chapters(
                &mut book,
                &mut bib,
                &config.bib_hb_html,
                config.order.to_owned(),
            );
        }

        let cited =
            Bibliography::expand_cite_references_in_book(&mut book, &mut bib, &config.cite_hb_html);

        let bib_content_html = Bibliography::generate_bibliography_html(
            &bib,
            &cited,
            config.cited_only,
            config.bib_hb_html,
            config.order,
        );

        let bib_chapter = Bibliography::create_bibliography_chapter(
            config.title,
            config.js_html,
            config.css_html,
            bib_content_html,
        );

        book.push_item(bib_chapter);

        Ok(book)
    }

    fn supports_renderer(&self, renderer: &str) -> Result<bool, anyhow::Error> {
        Ok(renderer != "not-supported")
    }
}

fn add_bib_at_end_of_chapters(
    book: &mut Book,
    bibliography: &mut IndexMap<String, BibItem>,
    references_tpl: &String,
    order: SortOrder,
) {
    use regex::Regex;

    use std::collections::HashSet;

    lazy_static! {
        static ref REF_REGEX: Regex = Regex::new(REF_PATTERN).unwrap();
        static ref AT_REF_REGEX: Regex = Regex::new(AT_REF_PATTERN).unwrap();
    }

    book.for_each_mut(|section: &mut BookItem| {
        if let BookItem::Chapter(ref mut ch) = *section {
            if let Some(ref chapter_path) = ch.path {
                info!(
                    "Adding bibliography at the end of chapter {}",
                    chapter_path.as_path().display()
                );

                let mut cited = HashSet::new();
                // Find all {{#cite ...}} keys
                for caps in REF_REGEX.captures_iter(&ch.content) {
                    if let Some(cite) = caps.get(2) {
                        cited.insert(cite.as_str().trim().to_owned());
                    }
                }
                // Find all @@... keys
                for caps in AT_REF_REGEX.captures_iter(&ch.content) {
                    if let Some(cite) = caps.get(2) {
                        cited.insert(cite.as_str().trim().to_owned());
                    }
                }
                info!("Refs cited in this chapter: {cited:?}");

                let mut handlebars = Handlebars::new();
                handlebars
                    .register_template_string(
                        "chapter_refs",
                        config::DEFAULT_CHAPTER_REFS_FOOTER_HB_TEMPLATE,
                    )
                    .unwrap();

                let ch_bib_header_html = handlebars
                    .render("chapter_refs", &String::new())
                    .unwrap()
                    .as_str()
                    .to_string();

                let ch_bib_content_html = Bibliography::generate_bibliography_html(
                    bibliography,
                    &cited,
                    true,
                    references_tpl.to_string(),
                    order.clone(),
                );

                let new_content = String::from(ch.content.as_str())
                    + ch_bib_header_html.as_str()
                    + ch_bib_content_html.as_str();
                ch.content = new_content;
            }
        }
    });
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
    debug!("Handlebars content: {handlebars:?}");

    let chapter_path = chapter
        .path
        .as_deref()
        .unwrap_or_else(|| std::path::Path::new(""));

    lazy_static! {
        static ref REF_REGEX: Regex = Regex::new(REF_PATTERN).unwrap();
        static ref AT_REF_REGEX: Regex = Regex::new(AT_REF_PATTERN).unwrap();
    }

    // Wrap mutable state in RefCell for interior mutability
    let bib = RefCell::new(bibliography);
    let cited_set = RefCell::new(cited);
    let idx = RefCell::new(last_index);

    // First replace all {{#cite ...}}
    let replaced = REF_REGEX.replace_all(&chapter.content, |caps: &regex::Captures| {
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
    });

    replaced.into_owned()
}

// Regex patterns for citation placeholders
const REF_PATTERN: &str = r"
(?x)                       # insignificant whitespace mode
\\\{\{\#.*\}\}               # match escaped placeholder
|                            # or
\{\{\s*                      # placeholder opening parens and whitespace
\#([a-zA-Z0-9_]+)            # placeholder type
\s+                          # separating whitespace
([a-zA-Z0-9\s_.\-:/\\\+]+)   # placeholder target path and space separated properties
\s*\}\}                      # whitespace and placeholder closing parens";

const AT_REF_PATTERN: &str = r##"(@@)([^\[\]\s,;"#'()={}%]+)"##;

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
