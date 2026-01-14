use anyhow::anyhow;
use std::error::Error as StdError;
use std::ffi::OsString;
use std::fmt::{Display, Formatter};
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::str::FromStr;
use toml::value::Table;
use toml::Value;
use tracing::info;

use crate::backend::BackendMode;

/// Helper to extract a string from a TOML value with a descriptive error.
fn value_as_str<'a>(value: &'a Value, field_name: &str) -> Result<&'a str, anyhow::Error> {
    value
        .as_str()
        .ok_or_else(|| anyhow!("'{field_name}' must be a string"))
}

/// Helper to extract a bool from a TOML value with a descriptive error.
fn value_as_bool(value: &Value, field_name: &str) -> Result<bool, anyhow::Error> {
    value
        .as_bool()
        .ok_or_else(|| anyhow!("'{field_name}' must be a boolean"))
}

/// Helper to convert OsString to String with a descriptive error.
fn os_string_to_string(os: OsString) -> Result<String, anyhow::Error> {
    os.into_string()
        .map_err(|p| anyhow!("Path contains invalid UTF-8: {p:?}"))
}

pub static DEFAULT_JS_TEMPLATE: &str = include_str!("./render/copy2clipboard.js");
pub static DEFAULT_CSS_TEMPLATE: &str = include_str!("./render/satancisco.css");
pub static DEFAULT_HB_TEMPLATE: &str = include_str!("./render/references.hbs");
pub static DEFAULT_CITE_HB_TEMPLATE: &str = include_str!("./render/cite_key.hbs");
pub static DEFAULT_CHAPTER_REFS_FOOTER_HB_TEMPLATE: &str =
    include_str!("./render/chapter_refs_header.hbs");

type Error = anyhow::Error;

/// Error type for failed parsing of `String`s to `enum`s.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseEnumError(String);

impl Display for ParseEnumError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl StdError for ParseEnumError {}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SortOrder {
    None,
    Key,
    Author,
    Index,
}

impl FromStr for SortOrder {
    type Err = ParseEnumError;
    fn from_str(input: &str) -> Result<SortOrder, Self::Err> {
        match input {
            "none" => Ok(SortOrder::None),
            "key" => Ok(SortOrder::Key),
            "author" => Ok(SortOrder::Author),
            "index" => Ok(SortOrder::Index),
            _ => Err(ParseEnumError(format!(
                "Unknown option '{input}' for bibliograph order. Must be one of [none key author index]",
            ))),
        }
    }
}

#[derive(Debug)]
pub struct Config<'a> {
    /// Title for the Bibliography section of the book
    pub title: String,
    /// Path to Bibtex file
    pub bibliography: Option<&'a str>,
    /// Whether to add or not the bibliography at the end of each chapter too
    pub add_bib_in_each_chapter: bool,
    /// Zotero user ID, as alternative to Bibtex file
    pub zotero_uid: Option<&'a str>,
    /// List only cited references, instead of all from bibliography
    pub cited_only: bool,
    /// HTML content of the Handlebars render template for references
    pub bib_hb_html: String,
    /// HTML content of the Handlebars render template for inline citations
    pub cite_hb_html: String,
    /// Extra CSS style content for the ad-hoc Handlebars template
    pub css_html: String,
    /// Extra Javascript functions for the ad-hoc Handlebars template
    pub js_html: String,
    /// Sort order in bibliography output
    pub order: SortOrder,
    /// Backend mode: Custom (Handlebars) or CSL
    pub backend: BackendMode,
    /// CSL style name (only used when backend = CSL)
    pub csl_style: Option<String>,
}

impl<'a> Config<'a> {
    pub fn build_from(table: Option<&'a Table>, book_src_path: PathBuf) -> Result<Self, Error> {
        let Some(table) = table else {
            return Err(anyhow!("No configuration provided."));
        };

        let title = match table.get("title") {
            Some(v) => value_as_str(v, "title")?.to_string(),
            None => "Bibliography".to_string(),
        };

        let bibliography = table
            .get("bibliography")
            .map(|v| value_as_str(v, "bibliography"))
            .transpose()?;

        let add_bib_in_each_chapter = match table.get("add-bib-in-chapters") {
            None => false,
            Some(v) => value_as_bool(v, "add-bib-in-chapters")?,
        };

        let zotero_uid = table
            .get("zotero-uid")
            .map(|v| value_as_str(v, "zotero-uid"))
            .transpose()?;

        let cited_only = match table.get("render-bib") {
            None => true,
            Some(v) => match value_as_str(v, "render-bib")? {
                "cited" => true,
                "all" => false,
                other => {
                    return Err(anyhow!(
                        "Unknown value '{other}' for option 'render-bib'. \
                        Use one of [cited, all]. Skipping bibliography."
                    ));
                }
            },
        };

        let bib_hb_html = match table.get("hb-tpl") {
            Some(template) => {
                let template_path =
                    book_src_path.join(Path::new(value_as_str(template, "hb-tpl")?));
                let template_path_str = os_string_to_string(template_path.into_os_string())?;
                info!("Using HB template for bibliography from {template_path_str:?}...");
                let template_content = fs::read_to_string(template_path_str)?;
                format!("\n\n{template_content}\n\n")
            }
            None => {
                info!("Using default HB template...");
                format!("\n\n{DEFAULT_HB_TEMPLATE}\n\n")
            }
        };

        let cite_hb_html = match table.get("cite-hb-tpl") {
            Some(template) => {
                let template_path =
                    book_src_path.join(Path::new(value_as_str(template, "cite-hb-tpl")?));
                let template_path_str = os_string_to_string(template_path.into_os_string())?;
                info!("Using HB template for citations from {template_path_str:?}...");
                fs::read_to_string(template_path_str)?
            }
            None => {
                info!("Using default citation HB template...");
                DEFAULT_CITE_HB_TEMPLATE.to_string()
            }
        };

        let css_html = match table.get("css") {
            Some(css) => {
                let css_path = book_src_path.join(Path::new(value_as_str(css, "css")?));
                let css_path_str = os_string_to_string(css_path.into_os_string())?;
                info!("Using CSS style for bibliography from {css_path_str:?}...");
                let css_content = fs::read_to_string(css_path_str)?;
                format!("<style>{css_content}</style>\n\n")
            }
            None => {
                info!("Using default CSS template...");
                format!("<style>{DEFAULT_CSS_TEMPLATE}</style>\n\n")
            }
        };

        let js_html = match table.get("js") {
            Some(js) => {
                let js_path = book_src_path.join(Path::new(value_as_str(js, "js")?));
                let js_path_str = os_string_to_string(js_path.into_os_string())?;
                info!("Using JS template for bibliography from {js_path_str:?}...");
                let js_content = fs::read_to_string(js_path_str)?;
                format!("<script type=\"text/javascript\">\n{js_content}\n</script>\n\n")
            }
            None => {
                info!("Using default JS template...");
                format!("<script type=\"text/javascript\">\n{DEFAULT_JS_TEMPLATE}\n</script>\n\n")
            }
        };

        let order = match table.get("order") {
            Some(v) => SortOrder::from_str(value_as_str(v, "order")?)?,
            None => SortOrder::None,
        };

        let backend = match table.get("backend") {
            Some(v) => match value_as_str(v, "backend")? {
                "custom" => {
                    info!("Using Custom (Handlebars) backend");
                    BackendMode::Custom
                }
                "csl" => {
                    info!("Using CSL backend");
                    BackendMode::Csl
                }
                other => {
                    return Err(anyhow!(
                        "Unknown backend '{other}'. Use one of [custom, csl]. \
                        Defaulting to 'custom'."
                    ));
                }
            },
            None => {
                info!("No backend specified, defaulting to Custom (Handlebars)");
                BackendMode::Custom
            }
        };

        let csl_style = table
            .get("csl-style")
            .map(|v| value_as_str(v, "csl-style").map(|s| s.to_string()))
            .transpose()?;

        Ok(Self {
            title,
            bibliography,
            add_bib_in_each_chapter,
            zotero_uid,
            cited_only,
            bib_hb_html,
            cite_hb_html,
            css_html,
            js_html,
            order,
            backend,
            csl_style,
        })
    }
}
