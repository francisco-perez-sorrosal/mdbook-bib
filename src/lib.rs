use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context};
use handlebars::Handlebars;
use mdbook_preprocessor::book::{Book, Chapter};
use mdbook_preprocessor::errors::Error;
use mdbook_preprocessor::{Preprocessor, PreprocessorContext};

mod backend;
mod citation;
mod config;
mod file_utils;
mod io;
mod models;
mod parser;
mod renderer;

use crate::backend::{BackendMode, CslBackend, CustomBackend};
use crate::config::Config;
use crate::parser::BibFormat;

// Re-export for tests
#[cfg(test)]
pub use citation::{AT_REF_PATTERN, REF_PATTERN};

static NAME: &str = "bib";
static BIB_OUT_FILE: &str = "bibliography";

pub struct Bibliography;

impl Default for Bibliography {
    fn default() -> Bibliography {
        Bibliography
    }
}

impl Bibliography {
    fn retrieve_bibliography_content(
        ctx: &PreprocessorContext,
        cfg: &Config,
    ) -> Result<(String, BibFormat), Error> {
        let (bib_content, format) = match &cfg.bibliography {
            Some(biblio_file) => {
                tracing::info!("Bibliography file: {}", biblio_file);
                let mut biblio_path = ctx.root.join(&ctx.config.book.src);
                biblio_path = biblio_path.join(Path::new(&biblio_file));
                if !biblio_path.exists() {
                    Err(anyhow!("Bibliography file {biblio_path:?} not found!",))
                } else {
                    tracing::info!("Bibliography path: {}", biblio_path.display());
                    let format = io::detect_format(&biblio_path);
                    let content = io::load_bibliography(biblio_path)?;
                    Ok((content, format))
                }
            }
            _ => {
                tracing::warn!("Bibliography file not specified. Trying download from Zotero");
                match &cfg.zotero_uid {
                    Some(uid) => {
                        let user_id = uid.to_string();
                        let bib_str = io::download_bib_from_zotero(user_id).unwrap_or_default();
                        if !bib_str.is_empty() {
                            let biblio_path = ctx.root.join(Path::new("my_zotero.bib"));
                            tracing::info!("Saving Zotero bibliography to {:?}", biblio_path);
                            let _ = fs::write(biblio_path, &bib_str);
                            // Zotero always returns BibTeX format
                            Ok((bib_str, BibFormat::BibTeX))
                        } else {
                            Err(anyhow!("Bib content retrieved from Zotero is empty!"))
                        }
                    }
                    _ => Err(anyhow!("Zotero user id not specified either :(")),
                }
            }
        }?;
        Ok((bib_content, format))
    }

    fn create_bibliography_chapter(
        title: String,
        js_html_part: String,
        css_html_part: String,
        biblio_html_part: String,
    ) -> Chapter {
        let html_content = format!("{js_html_part}\n{css_html_part}\n{biblio_html_part}");
        tracing::debug!(
            "Creating new Bibliography chapter (with title: \"{}\") with content: {:?}",
            title,
            html_content
        );

        Chapter::new(
            &title,
            format!("# {title}\n{html_content}"),
            PathBuf::from(format!("{BIB_OUT_FILE}.md")),
            Vec::new(),
        )
    }
}

impl Preprocessor for Bibliography {
    fn name(&self) -> &str {
        NAME
    }

    fn run(&self, ctx: &PreprocessorContext, mut book: Book) -> Result<Book, anyhow::Error> {
        tracing::info!("Processor Name: {}", self.name());
        let book_src_root = ctx.root.join(&ctx.config.book.src);
        let table = match ctx.config.get::<toml::value::Table>("preprocessor.bib") {
            Ok(Some(table)) => Some(table),
            Ok(None) => {
                tracing::warn!("No [preprocessor.bib] section found. Skipping processing.");
                return Ok(book);
            }
            Err(err) => {
                tracing::warn!("Error reading configuration. Skipping processing: {err:?}");
                return Ok(book);
            }
        };
        let config = match Config::build_from(table.as_ref(), book_src_root) {
            Ok(config) => config,
            Err(err) => {
                tracing::warn!(
                    "Error reading configuration. Skipping processing: {:?}",
                    err
                );
                return Ok(book);
            }
        };

        // Configure template registry
        let mut handlebars = Handlebars::new();
        handlebars
            .register_template_string("references", &config.bib_hb_html)
            .context("Failed to register references template. Check your 'hb-tpl' configuration for valid Handlebars syntax")?;
        handlebars
            .register_template_string(
                "chapter_refs",
                config::DEFAULT_CHAPTER_REFS_FOOTER_HB_TEMPLATE,
            )
            .context(
                "Failed to register chapter_refs template. This is a bug - please report it",
            )?;
        handlebars
            .register_template_string("citation", &config.cite_hb_html)
            .context("Failed to register citation template. Check your 'cite-hb-tpl' configuration for valid Handlebars syntax")?;
        tracing::debug!("Handlebars content: {:?}", handlebars);

        let bib_result = Bibliography::retrieve_bibliography_content(ctx, &config);

        if bib_result.is_err() {
            tracing::warn!(
                "Raw Bibliography content couldn't be retrieved. Skipping processing: {:?}",
                bib_result.err()
            );
            return Ok(book);
        }

        let (bib_content, format) = bib_result?;

        let bibliography = parser::parse_bibliography(bib_content, format);
        if bibliography.is_err() {
            tracing::warn!(
                "Error building Bibliography from raw content. Skipping render: {:?}",
                bibliography.err()
            );
            return Ok(book);
        }

        let mut bib = bibliography.unwrap();

        // Create the appropriate backend based on configuration
        let backend: Box<dyn crate::backend::BibliographyBackend> = match config.backend {
            BackendMode::Custom => {
                tracing::info!("Using Custom (Handlebars) backend for rendering");
                Box::new(CustomBackend::new(&handlebars))
            }
            BackendMode::Csl => {
                tracing::info!(
                    "Using CSL backend with style '{}'",
                    config.csl_style.as_deref().unwrap_or("apa")
                );
                let style = config.csl_style.as_deref().unwrap_or("apa");
                Box::new(
                    CslBackend::new(style.to_string())
                        .context("Failed to initialize CSL backend")?,
                )
            }
        };

        tracing::info!("Backend initialized: {}", backend.name());

        // First, expand citations to assign indices to BibItems
        let citation_result =
            citation::expand_cite_references_in_book(&mut book, &mut bib, backend.as_ref());

        // Then add per-chapter bibliographies (now items have correct indices)
        if config.add_bib_in_each_chapter {
            let chapter_refs_header = handlebars
                .render("chapter_refs", &String::new())
                .context("Failed to render chapter_refs header")?;

            citation::add_bib_at_end_of_chapters(
                &mut book,
                &mut bib,
                backend.as_ref(),
                &chapter_refs_header,
                config.order.clone(),
                &citation_result.per_chapter,
            );
        }

        let bib_content_html = renderer::generate_bibliography_html(
            &bib,
            &citation_result.all_cited,
            config.cited_only,
            backend.as_ref(),
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

#[cfg(test)]
mod tests;
