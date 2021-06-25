use anyhow::anyhow;
use std::convert::TryFrom;
use toml::value::Table;

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
}

impl<'a> TryFrom<Option<&'a Table>> for Config<'a> {
    type Error = anyhow::Error;

    fn try_from(table: Option<&'a Table>) -> Result<Self, Self::Error> {
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
            })
        } else {
            Err(anyhow!("No configuration provided."))
        }
    }
}
