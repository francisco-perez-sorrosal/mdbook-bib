use anyhow::anyhow;
use std::convert::TryFrom;
use toml::value::Table;

#[derive(Debug)]
pub struct Config<'a> {
    /// Path to Bibtex file
    pub bibliography: Option<&'a str>,
    /// Zotero user ID, as alternative to Bibtex file
    pub zotero_user_id: Option<&'a str>,
    /// List only cited references, instead of all from bibliography
    pub cited_only: bool,
}

impl<'a> TryFrom<Option<&'a Table>> for Config<'a> {
    type Error = anyhow::Error;

    fn try_from(table: Option<&'a Table>) -> Result<Self, Self::Error> {
        if let Some(table) = table {
            Ok(Self {
                bibliography: table.get("bibliography").map(|v| v.as_str().unwrap()),

                zotero_user_id: table.get("zotero_user_id").map(|v| v.as_str().unwrap()),

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
