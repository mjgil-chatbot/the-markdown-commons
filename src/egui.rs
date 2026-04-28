//! Repo-owned markdown renderer with improved table support.
//!
//! This module avoids `egui_commonmark` entirely. It renders markdown blocks
//! with explicit egui layout so message rendering stays deterministic.
//!
//! Extracted from `rust-chatbot/src/markdown.rs`.

#[cfg(test)]
mod headless_layout_tests;
#[cfg(test)]
mod headless_tests;
mod inline;
mod lists;
#[cfg(test)]
mod parse;
mod render;
mod segments;
mod style;
#[allow(unused_imports)]
use crate::common::{self, Alignment};
/// A table extracted from markdown for custom rendering.
///
/// `MarkdownTable` holds the raw string content of table cells without
/// inline formatting applied. Use `parse_markdown_segments` to extract
/// tables from markdown content.
#[derive(Debug, Clone)]
pub struct MarkdownTable {
    /// Table header column names.
    pub headers: Vec<String>,
    /// Column alignment hints (currently unused in egui rendering).
    #[allow(dead_code)] // Reserved for future column alignment support
    pub alignments: Vec<crate::common::Alignment>,
    /// Table data rows.
    pub rows: Vec<Vec<String>>,
}
/// Segments produced by parsing markdown content.
///
/// `MarkdownSegment` represents either plain text or a table extracted
/// from markdown. Use `parse_markdown_segments` to split mixed content.
#[derive(Debug)]
pub enum MarkdownSegment {
    /// Plain text content with inline formatting.
    Text(String),
    /// A parsed table.
    Table(MarkdownTable),
}
