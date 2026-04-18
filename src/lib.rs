//! Shared markdown parsing and rendering for HTML and egui.
//!
//! - `parse_document` is the primary typed API
//! - `parse_markdown*` helpers are convenience HTML wrappers
//! - `DocumentCache` provides reusable parsed-document caching

mod common;
pub mod document;
pub mod inline;
mod parse_util;
#[cfg(feature = "egui")]
pub mod egui;
