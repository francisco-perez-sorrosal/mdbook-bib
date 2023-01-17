use anyhow::anyhow;
use log::info;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use toml::value::Table;

pub static DEFAULT_JS_TEMPLATE: &str = include_str!("./render/copy2clipboard.js");
pub static DEFAULT_CSS_TEMPLATE: &str = include_str!("./render/satancisco.css");
pub static DEFAULT_HB_TEMPLATE: &str = include_str!("./render/references.hbs");
pub static DEFAULT_CITE_HB_TEMPLATE: &str = include_str!("./render/cite_key.hbs");

#[derive(Debug)]
pub struct Config<'a> {
    /// Title for the Bibliography section of the book
    pub title: String,
    /// Path to Bibtex file
    pub bibliography: Option<&'a str>,
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
}

type Error = anyhow::Error;

impl<'a> Config<'a> {
    pub fn build_from(table: Option<&'a Table>, book_src_path: PathBuf) -> Result<Self, Error> {
        if let Some(table) = table {
            Ok(Self {
                title: match table.get("title") {
                    Some(bib_title) => bib_title.as_str().unwrap().to_string(),
                    None => "Bibliography".to_string(),
                },

                bibliography: table.get("bibliography").map(|v| v.as_str().unwrap()),
                zotero_uid: table.get("zotero-uid").map(|v| v.as_str().unwrap()),

                cited_only: match table.get("render-bib") {
                    None => true,
                    Some(option) => match option.as_str().unwrap() {
                        "cited" => true,
                        "all" => false,
                        other => {
                            return Err(anyhow!(
                                "Unknown value '{}' for option 'render-bib'. \
                                Use one of [cited, all]. Skipping bibliography.",
                                other
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
                        info!(
                            "Using HB template for bibliography from {:?}...",
                            template_path_str
                        );
                        let template_content = fs::read_to_string(template_path_str)?;
                        format!("\n\n{}\n\n", template_content)
                    }
                    None => {
                        info!("Using default HB template...");
                        format!("\n\n{}\n\n", DEFAULT_HB_TEMPLATE)
                    }
                },

                cite_hb_html: match table.get("cite-hb-tpl") {
                    Some(template) => {
                        let template_path =
                            book_src_path.join(Path::new(&template.as_str().unwrap().to_string()));
                        let template_path_str =
                            template_path.into_os_string().into_string().unwrap();
                        info!(
                            "Using HB template for citations from {:?}...",
                            template_path_str
                        );
                        let template_content = fs::read_to_string(template_path_str)?;
                        format!("\n\n{}\n\n", template_content)
                    }
                    None => {
                        info!("Using default citation HB template...");
                        format!("\n\n{}\n\n", DEFAULT_CITE_HB_TEMPLATE)
                    }
                },

                css_html: match table.get("css") {
                    Some(css) => {
                        let css_path =
                            book_src_path.join(Path::new(&css.as_str().unwrap().to_string()));
                        let css_path_str = css_path.into_os_string().into_string().unwrap();
                        info!(
                            "Using CSS style for bibliography from {:?}...",
                            css_path_str
                        );
                        let css_content = fs::read_to_string(css_path_str)?;
                        format!("<style>{}</style>\n\n", css_content)
                    }
                    None => {
                        info!("Using default CSS template...");
                        format!("<style>{}</style>\n\n", DEFAULT_CSS_TEMPLATE) // Add the style css for the biblio
                    }
                },

                js_html: match table.get("js") {
                    Some(css) => {
                        let js_path =
                            book_src_path.join(Path::new(&css.as_str().unwrap().to_string()));
                        let js_path_str = js_path.into_os_string().into_string().unwrap();
                        info!(
                            "Using JS template for bibliography from {:?}...",
                            js_path_str
                        );
                        let js_content = fs::read_to_string(js_path_str)?;
                        format!(
                            "<script type=\"text/javascript\">\n{}\n</script>\n\n",
                            js_content
                        )
                    }
                    None => {
                        info!("Using default JS template...");
                        format!(
                            "<script type=\"text/javascript\">\n{}\n</script>\n\n",
                            DEFAULT_JS_TEMPLATE
                        )
                    }
                },
            })
        } else {
            Err(anyhow!("No configuration provided."))
        }
    }
}
