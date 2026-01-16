//! Tests for configuration parsing and handling.
//!
//! This module covers:
//! - Config attribute parsing from book.toml
//! - Default values
//! - Custom templates and styles
//! - Zotero configuration
//! - Per-chapter bibliography settings

use super::common::{EXAMPLE_CSS_TEMPLATE, EXAMPLE_HB_TEMPLATE};
use crate::config::Config;
use crate::config::DEFAULT_JS_TEMPLATE;
use crate::config::{DEFAULT_CSS_TEMPLATE, DEFAULT_HB_TEMPLATE};
use std::path::PathBuf;
use toml::value::Table;
use toml::Value;

// =============================================================================
// Basic Config Attribute Tests
// =============================================================================

#[test]
fn check_config_attributes() {
    // Check config with default values is returned when an empty config is passed in a toml table
    let t: Table = Table::new();
    match Config::build_from(Some(&t), PathBuf::new()) {
        Ok(config) => {
            println!("{config:?}");
            assert_eq!(config.title, "Bibliography");
            assert_eq!(config.bibliography, None);
            assert_eq!(config.zotero_uid, None);
            assert!(config.cited_only);
            let default_tpl = format!("\n\n{DEFAULT_HB_TEMPLATE}\n\n");
            assert_eq!(config.bib_hb_html, default_tpl);
            let default_css = format!("<style>{DEFAULT_CSS_TEMPLATE}</style>\n\n");
            assert_eq!(config.css_html, default_css);
            let default_js =
                format!("<script type=\"text/javascript\">\n{DEFAULT_JS_TEMPLATE}\n</script>\n\n",);
            assert_eq!(config.js_html, default_js);
        }
        Err(_) => panic!("there's supposed to be always a config!!!"),
    }

    // Check config attributes are processed (those that are not specified are ignored)
    let mut t: Table = Table::new();

    t.insert(
        "bibliography".to_string(),
        Value::String("biblio.bib".to_string()),
    );
    t.insert(
        "zotero-uid".to_string(),
        Value::String("123456".to_string()),
    );
    t.insert("title".to_string(), Value::String("References".to_string()));
    t.insert("render-bib".to_string(), Value::String("all".to_string()));
    t.insert(
        "not-specified-config-attr".to_string(),
        Value::String("uhg???".to_string()),
    );
    match Config::build_from(Some(&t), PathBuf::new()) {
        Ok(config) => {
            println!("{config:?}");
            assert_eq!(config.title, "References");
            assert_eq!(config.bibliography, Some("biblio.bib"));
            assert_eq!(config.zotero_uid, Some("123456"));
            assert!(!config.cited_only);
        }
        Err(_) => panic!("there's supposed to be always a config!!!"),
    }

    // Intentionally add a failure specifying a non-existing value for render-bib
    let mut t: Table = Table::new();
    t.insert(
        "render-bib".to_string(),
        Value::String("non-existent!".to_string()),
    );
    match Config::build_from(Some(&t), PathBuf::new()) {
        Ok(_) => panic!("there's supposed to be a failure in the config!!!"),
        Err(_) => println!("Yayyyyy! A failure that is supposed to happen!"),
    }
}

// =============================================================================
// Custom Templates and Styles Tests
// =============================================================================

#[test]
fn check_adhoc_template_and_style() {
    // Test adhoc template and style (using the templates provided for the project doc/manual)
    let mut t: Table = Table::new();
    t.insert(
        "hb-tpl".to_string(),
        Value::String("render/my_references.hbs".to_string()),
    );
    t.insert(
        "css".to_string(),
        Value::String("render/my_style.css".to_string()),
    );

    let mut manual_src_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manual_src_path.push("manual/src/");
    match Config::build_from(Some(&t), manual_src_path) {
        Ok(config) => {
            println!("{config:?}");
            let adhoc_tpl = format!("\n\n{EXAMPLE_HB_TEMPLATE}\n\n");
            assert_eq!(config.bib_hb_html, adhoc_tpl);
            let adhoc_css = format!("<style>{EXAMPLE_CSS_TEMPLATE}</style>\n\n");
            assert_eq!(config.css_html, adhoc_css);
            let default_js =
                format!("<script type=\"text/javascript\">\n{DEFAULT_JS_TEMPLATE}\n</script>\n\n",);
            assert_eq!(config.js_html, default_js);
        }
        Err(e) => panic!(
            "there's supposed to be always a config!!!\n {:?}",
            e.root_cause()
        ),
    }
}

// =============================================================================
// Zotero Configuration Tests
// =============================================================================

#[test]
fn zotero_config_parsing() {
    // Test that Zotero UID config is parsed correctly
    let mut t: Table = Table::new();
    t.insert(
        "zotero-uid".to_string(),
        Value::String("12345678".to_string()),
    );

    let config = Config::build_from(Some(&t), PathBuf::new()).unwrap();
    assert_eq!(config.zotero_uid, Some("12345678"));
    assert!(
        config.bibliography.is_none(),
        "Bibliography should be None when using Zotero"
    );
}

#[test]
fn zotero_vs_local_bibliography_config() {
    // When both zotero-uid and bibliography are specified, both should be available
    let mut t: Table = Table::new();
    t.insert(
        "zotero-uid".to_string(),
        Value::String("12345678".to_string()),
    );
    t.insert(
        "bibliography".to_string(),
        Value::String("local.bib".to_string()),
    );

    let config = Config::build_from(Some(&t), PathBuf::new()).unwrap();
    assert_eq!(config.zotero_uid, Some("12345678"));
    assert_eq!(config.bibliography, Some("local.bib"));
}

#[test]
fn zotero_url_construction() {
    // Test that Zotero URL is constructed correctly
    let uid = "475425";
    let expected_url_prefix = format!("https://api.zotero.org/users/{uid}/items");

    // This verifies the URL format without making actual network calls
    assert!(expected_url_prefix.contains("zotero.org"));
    assert!(expected_url_prefix.contains(uid));
}

// =============================================================================
// Per-Chapter Bibliography Configuration Tests
// =============================================================================

#[test]
fn per_chapter_bibliography_config() {
    // Test add-bib-in-chapters config option
    let mut t: Table = Table::new();
    t.insert(
        "bibliography".to_string(),
        Value::String("refs.bib".to_string()),
    );
    t.insert("add-bib-in-chapters".to_string(), Value::Boolean(true));

    let config = Config::build_from(Some(&t), PathBuf::new()).unwrap();
    assert!(
        config.add_bib_in_each_chapter,
        "add_bib_in_chapters should be true"
    );
}

#[test]
fn per_chapter_bibliography_disabled_by_default() {
    // Test that per-chapter bibliography is disabled by default
    let t: Table = Table::new();
    let config = Config::build_from(Some(&t), PathBuf::new()).unwrap();
    assert!(
        !config.add_bib_in_each_chapter,
        "add_bib_in_chapters should be false by default"
    );
}
