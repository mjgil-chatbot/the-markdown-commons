use crate::common::Alignment;
use crate::inline::InlineNode;
use std::collections::{HashMap, VecDeque};
mod cache;
mod markdownish;
/// A fully parsed markdown document, ready for rendering.
///
/// `MarkdownDocument` is the shared intermediate representation produced by the
/// parser and consumed by both the HTML and egui renderers. It contains:
///
/// - `blocks`: the parsed document structure (headings, paragraphs, lists, etc.)
/// - `footnotes`: all footnote definitions found in the document
/// - `references`: all link reference definitions found in the document
///
/// # Example
///
/// ```
/// use tmc::{parse_document, Block};
///
/// let doc = parse_document("# Title\n\nSome text");
/// assert!(matches!(doc.blocks[0], Block::Heading(_)));
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct MarkdownDocument {
    /// The top-level blocks in document order.
    pub blocks: Vec<Block>,
    /// All footnote definitions found in the document.
    pub footnotes: Vec<FootnoteDefinition>,
    /// All link reference definitions found in the document.
    pub references: Vec<LinkReferenceDefinition>,
}
/// A cache for parsed markdown documents that avoids re-parsing the same content.
///
/// `DocumentCache` stores parsed documents keyed by their input text. When
/// the same markdown is parsed again, the cached result is returned immediately.
///
/// The cache maintains separate storage for full documents and fragments,
/// and uses LRU-style ordering to bound memory usage.
///
/// # Example
///
/// ```
/// use tmc::{DocumentCache, parse_markdown_cached};
///
/// let mut cache = DocumentCache::default();
///
/// // First call parses and caches
/// let html1 = parse_markdown_cached(&mut cache, "# Hello");
///
/// // Second call with same input hits cache
/// let html2 = parse_markdown_cached(&mut cache, "# Hello");
///
/// // Cache can be cleared to free memory
/// cache.clear();
/// ```
#[derive(Debug, Default, Clone)]
pub struct DocumentCache {
    full_documents: HashMap<String, MarkdownDocument>,
    fragment_documents: HashMap<String, MarkdownDocument>,
    full_order: VecDeque<String>,
    fragment_order: VecDeque<String>,
    full_document_bytes: usize,
    fragment_document_bytes: usize,
}
/// The top-level block-level nodes in a parsed markdown document.
///
/// Each variant represents a different markdown construct. Use `matches!` or
/// `if let` to handle specific block types:
///
/// ```
/// use tmc::{Block, parse_document};
///
/// let doc = parse_document("# Hello\n\nParagraph");
/// for block in &doc.blocks {
///     match block {
///         Block::Heading(h) => println!("Heading level {}", h.level),
///         Block::Paragraph(_) => println!("Paragraph"),
///         _ => {}
///     }
/// }
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Block {
    /// A paragraph of text.
    Paragraph(ParagraphBlock),
    /// A heading (ATX or setext style).
    Heading(HeadingBlock),
    /// A fenced or indented code block.
    CodeBlock(CodeBlock),
    /// A table with headers and rows.
    Table(TableBlock),
    /// A blockquote containing nested blocks.
    BlockQuote(Vec<Block>),
    /// A definition list with terms and definitions.
    DefinitionList(Vec<DefinitionItem>),
    /// A task list with checkboxes.
    TaskList(Vec<TaskItem>),
    /// An unordered (bulleted) list.
    UnorderedList(UnorderedListBlock),
    /// An ordered (numbered) list.
    OrderedList(OrderedListBlock),
    /// A horizontal rule separator.
    HorizontalRule,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnorderedListBlock {
    pub indent: usize,
    pub items: Vec<ListItem>,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OrderedListBlock {
    pub start: usize,
    pub indent: usize,
    pub items: Vec<ListItem>,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParagraphBlock {
    pub lines: Vec<ParagraphLine>,
    pub content: InlineContent,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParagraphLine {
    pub content: InlineContent,
    pub hard_break: bool,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HeadingBlock {
    pub level: usize,
    pub content: InlineContent,
    pub id: Option<String>,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CodeBlock {
    pub language: Option<String>,
    pub code: String,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TableBlock {
    pub headers: Vec<InlineContent>,
    pub alignments: Vec<Option<Alignment>>,
    pub rows: Vec<Vec<InlineContent>>,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DefinitionItem {
    pub term: InlineContent,
    pub definitions: Vec<InlineContent>,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskItem {
    pub checked: bool,
    pub content: InlineContent,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ListItem {
    pub content: InlineContent,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InlineContent {
    pub source: String,
    pub nodes: Vec<InlineNode>,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FootnoteDefinition {
    pub id: String,
    pub content: String,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LinkReferenceDefinition {
    pub label: String,
    pub url: String,
    pub title: Option<String>,
}
