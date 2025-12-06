use anyhow::anyhow;
use log::info;
use std::error::Error as StdError;
use std::fmt::{Display, Formatter};
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::str::FromStr;
use toml::value::Table;

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
}

impl<'a> Config<'a> {
    pub fn build_from(table: Option<&'a Table>, book_src_path: PathBuf) -> Result<Self, Error> {
        if let Some(table) = table {
            Ok(Self {
                title: match table.get("title") {
                    Some(bib_title) => bib_title.as_str().unwrap().to_string(),
                    None => "Bibliography".to_string(),
                },

                bibliography: table.get("bibliography").map(|v| v.as_str().unwrap()),

                add_bib_in_each_chapter: match table.get("add-bib-in-chapters") {
                    None => false,
                    Some(option) => option.as_bool().unwrap(),
                },

                zotero_uid: table.get("zotero-uid").map(|v| v.as_str().unwrap()),

                cited_only: match table.get("render-bib") {
                    None => true,
                    Some(option) => match option.as_str().unwrap() {
                        "cited" => true,
                        "all" => false,
                        other => {
                            return Err(anyhow!(
                                "Unknown value '{other}' for option 'render-bib'. \
                                Use one of [cited, all]. Skipping bibliography."
                            ));
                        }
                    },
                },

                bib_hb_html: match table.get("hb-tpl") {
                    Some(template) => {
                        let template_path =
                            book_src_path.join(Path::new(&template.as_str().unwrap().to_string()));
                        let template_path_str =
                            template_path.into_os_string().into_string().unwrap();
                        info!("Using HB template for bibliography from {template_path_str:?}...");
                        let template_content = fs::read_to_string(template_path_str)?;
                        format!("\n\n{template_content}\n\n")
                    }
                    None => {
                        info!("Using default HB template...");
                        format!("\n\n{DEFAULT_HB_TEMPLATE}\n\n")
                    }
                },

                cite_hb_html: match table.get("cite-hb-tpl") {
                    Some(template) => {
                        let template_path =
                            book_src_path.join(Path::new(&template.as_str().unwrap().to_string()));
                        let template_path_str =
                            template_path.into_os_string().into_string().unwrap();
                        info!("Using HB template for citations from {template_path_str:?}...");
                        fs::read_to_string(template_path_str)?
                    }
                    None => {
                        info!("Using default citation HB template...");
                        DEFAULT_CITE_HB_TEMPLATE.to_string()
                    }
                },

                css_html: match table.get("css") {
                    Some(css) => {
                        let css_path =
                            book_src_path.join(Path::new(&css.as_str().unwrap().to_string()));
                        let css_path_str = css_path.into_os_string().into_string().unwrap();
                        info!("Using CSS style for bibliography from {css_path_str:?}...");
                        let css_content = fs::read_to_string(css_path_str)?;
                        format!("<style>{css_content}</style>\n\n")
                    }
                    None => {
                        info!("Using default CSS template...");
                        format!("<style>{DEFAULT_CSS_TEMPLATE}</style>\n\n") // Add the style css for the biblio
                    }
                },

                js_html: match table.get("js") {
                    Some(css) => {
                        let js_path =
                            book_src_path.join(Path::new(&css.as_str().unwrap().to_string()));
                        let js_path_str = js_path.into_os_string().into_string().unwrap();
                        info!("Using JS template for bibliography from {js_path_str:?}...");
                        let js_content = fs::read_to_string(js_path_str)?;
                        format!("<script type=\"text/javascript\">\n{js_content}\n</script>\n\n")
                    }
                    None => {
                        info!("Using default JS template...");
                        format!("<script type=\"text/javascript\">\n{DEFAULT_JS_TEMPLATE}\n</script>\n\n")
                    }
                },

                order: match table.get("order") {
                    Some(order) => SortOrder::from_str(order.as_str().unwrap())?,
                    None => SortOrder::None,
                },
            })
        } else {
            Err(anyhow!("No configuration provided."))
        }
    }
}
