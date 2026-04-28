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
pub use common::Alignment;
pub use document::{
    parse_document, parse_document_cached, parse_document_fragment, parse_document_fragment_cached,
    Block, CodeBlock, DefinitionItem, DocumentCache, FootnoteDefinition, HeadingBlock,
    InlineContent, LinkReferenceDefinition, ListItem, MarkdownDocument, OrderedListBlock,
    ParagraphBlock, ParagraphLine, TableBlock, TaskItem, UnorderedListBlock,
};
pub use html::{
    parse_markdown, parse_markdown_cached, parse_markdown_fragment, parse_markdown_fragment_cached,
    parse_markdown_fragment_with_options, parse_markdown_fragment_with_options_cached,
    parse_markdown_with_options, parse_markdown_with_options_cached, HtmlOptions,
};
pub use inline::{parse_inline_nodes, InlineNode, LinkDefinitions};

#[cfg(feature = "egui")]
pub mod egui;

#[cfg(feature = "egui")]
pub use egui::{
    render_markdown, render_markdown_with_options, MarkdownCache, MarkdownRenderOptions,
    MarkdownSegment, MarkdownTable,
};

#[cfg(test)]
mod cap_tests {
    use cap::Cap;
    use std::alloc;

    const TEST_ALLOCATION_LIMIT_BYTES: usize = 64 * 1024 * 1024;

    #[global_allocator]
    static TEST_ALLOCATOR: Cap<alloc::System> =
        Cap::new(alloc::System, TEST_ALLOCATION_LIMIT_BYTES);

    #[test]
    fn small_allocations_fit_under_test_cap() {
        let bytes: Vec<u8> = std::iter::repeat(0_u8).take(16 * 1024).collect();
        assert_eq!(bytes.len(), 16 * 1024);
    }
}
