//! Shared markdown parsing and rendering for HTML and egui.
//!
//! - `parse_document` is the primary typed API
//! - `parse_markdown*` helpers are convenience HTML wrappers
//! - `DocumentCache` provides reusable parsed-document caching

mod common;
pub mod document;
pub mod inline;
mod parse_util;
pub mod html;
pub use html::{
    parse_markdown, parse_markdown_cached, parse_markdown_fragment, parse_markdown_fragment_cached,
    parse_markdown_fragment_with_options, parse_markdown_fragment_with_options_cached,
    parse_markdown_with_options, parse_markdown_with_options_cached, HtmlOptions,
};
#[cfg(feature = "egui")]
pub mod egui;
