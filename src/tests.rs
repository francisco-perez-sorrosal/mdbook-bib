//! Test suite for mdbook-bib.
//!
//! This module organizes tests into logical submodules:
//!
//! - [`common`] - Shared test utilities, fixtures, and helpers
//! - [`parser`] - BibTeX and YAML parsing tests
//! - [`citation`] - Citation replacement and regex pattern tests
//! - [`config`] - Configuration parsing tests
//! - [`backend`] - Custom and CSL backend tests
//! - [`integration`] - Full book processing tests
//! - [`edge_cases`] - Error handling and edge case tests

#[cfg(test)]
mod common;

#[cfg(test)]
mod parser;

#[cfg(test)]
mod citation;

#[cfg(test)]
mod config;

#[cfg(test)]
mod backend;

#[cfg(test)]
mod integration;

#[cfg(test)]
mod edge_cases;
