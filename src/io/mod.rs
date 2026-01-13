use std::fs;
use std::io::Read;
use std::path::Path;

use anyhow::anyhow;
use mdbook_preprocessor::errors::{Error, Result as MdResult};
use reqwest::blocking::Response;

use crate::file_utils;

/// Load bibliography from file.
pub fn load_bibliography<P: AsRef<Path>>(biblio_file: P) -> MdResult<String> {
    tracing::info!("Loading bibliography from {:?}...", biblio_file.as_ref());

    let biblio_file_ext = file_utils::get_filename_extension(biblio_file.as_ref());
    if biblio_file_ext.unwrap_or_default().to_lowercase() != "bib" {
        tracing::warn!(
            "Only biblatex-based bibliography is supported for now! Yours: {:?}",
            biblio_file.as_ref()
        );
        return Ok(String::new());
    }
    Ok(fs::read_to_string(biblio_file)?)
}

/// Download bibliography from Zotero.
pub fn download_bib_from_zotero(user_id: String) -> MdResult<String, Error> {
    let mut url = format!("https://api.zotero.org/users/{user_id}/items?format=biblatex&style=biblatex&limit=100&sort=creator&v=3");
    tracing::info!("Zotero's URL biblio source:\n{url:?}");
    let mut res = reqwest::blocking::get(&url)?;
    if res.status().is_client_error() || res.status().is_server_error() {
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
            tracing::info!("Next biblio chunk URL:\n{:?}", url);
            res = reqwest::blocking::get(&url)?;
            let (new_link_str, new_bib_part) = extract_biblio_data_and_link_info(&mut res);
            link_str = new_link_str;
            bib_content.push_str(&new_bib_part);
        }
        Ok(bib_content)
    }
}

fn extract_biblio_data_and_link_info(res: &mut Response) -> (String, String) {
    let mut biblio_chunk = String::new();
    let _ = res.read_to_string(&mut biblio_chunk);
    let link_info_in_header = res.headers().get("link");
    tracing::debug!("Header Link content: {:?}", link_info_in_header);
    let link_info_as_str = link_info_in_header.unwrap().to_str();

    (link_info_as_str.unwrap().to_string(), biblio_chunk)
}
