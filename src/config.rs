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

/// Helper to load a template file or use a default, with optional content wrapping.
fn load_template(
    table: &Table,
    key: &str,
    book_src_path: &Path,
    default_content: &str,
    description: &str,
    wrapper: impl Fn(&str) -> String,
) -> Result<String, anyhow::Error> {
    let content = match table.get(key) {
        Some(template) => {
            let template_path = book_src_path.join(Path::new(value_as_str(template, key)?));
            let template_path_str = os_string_to_string(template_path.into_os_string())?;
            info!("Using {description} from path: {template_path_str:?}...");
            fs::read_to_string(template_path_str)?
        }
        None => {
            info!("Using default {description}...");
            default_content.to_string()
        }
    };
    Ok(wrapper(&content))
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

/// Citation syntax determines which patterns are recognized in markdown.
///
/// - `Default`: Recognizes `{{#cite key}}` and `@@key` (mdbook-bib native syntax)
/// - `Pandoc`: Additionally recognizes `@key`, `[@key]`, `[-@key]` for Pandoc compatibility
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum CitationSyntax {
    /// Default mdbook-bib syntax: `{{#cite key}}` and `@@key`
    #[default]
    Default,
    /// Pandoc-compatible syntax: adds `@key`, `[@key]`, `[-@key]`
    Pandoc,
}

impl FromStr for CitationSyntax {
    type Err = ParseEnumError;
    fn from_str(input: &str) -> Result<CitationSyntax, Self::Err> {
        match input {
            "default" => Ok(CitationSyntax::Default),
            "pandoc" => Ok(CitationSyntax::Pandoc),
            _ => Err(ParseEnumError(format!(
                "Unknown citation syntax '{input}'. Must be one of [default, pandoc]",
            ))),
        }
    }
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
    /// Citation syntax: which patterns are recognized in markdown
    pub citation_syntax: CitationSyntax,
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

        let bib_hb_html = load_template(
            table,
            "hb-tpl",
            &book_src_path,
            DEFAULT_HB_TEMPLATE,
            "HB template for bibliography",
            |c| format!("\n\n{c}\n\n"),
        )?;

        let cite_hb_html = load_template(
            table,
            "cite-hb-tpl",
            &book_src_path,
            DEFAULT_CITE_HB_TEMPLATE,
            "HB template for citations",
            |c| c.to_string(),
        )?;

        let css_html = load_template(
            table,
            "css",
            &book_src_path,
            DEFAULT_CSS_TEMPLATE,
            "CSS style for bibliography",
            |c| format!("<style>{c}</style>\n\n"),
        )?;

        let js_html = load_template(
            table,
            "js",
            &book_src_path,
            DEFAULT_JS_TEMPLATE,
            "JS for bibliography",
            |c| format!("<script type=\"text/javascript\">\n{c}\n</script>\n\n"),
        )?;

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

        let citation_syntax = match table.get("citation-syntax") {
            Some(v) => CitationSyntax::from_str(value_as_str(v, "citation-syntax")?)?,
            None => CitationSyntax::Default,
        };

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
            citation_syntax,
        })
    }
}
