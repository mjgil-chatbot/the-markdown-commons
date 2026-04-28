use crate::common::Alignment;
use crate::inline::InlineNode;
use std::collections::{HashMap, VecDeque};
use self::cache::{insert_cached_document, touch_cache_order};
mod cache;
mod markdownish;
mod parse_support;
pub(crate) mod parse {
    use crate::common;
    use crate::inline::{parse_inline_nodes, LinkDefinitions};
    use crate::parse_util::{normalize_reference_label, parse_destination_and_title};

    use super::parse_support::*;
    use super::{
        Block, CodeBlock, DefinitionItem, FootnoteDefinition, HeadingBlock, InlineContent,
        LinkReferenceDefinition, ListItem, MarkdownDocument, OrderedListBlock, ParagraphBlock,
        ParagraphLine, TableBlock, TaskItem, UnorderedListBlock,
    };

    pub(crate) fn parse_document(input: &str) -> MarkdownDocument {
        parse_document_inner(input)
    }

    pub(crate) fn parse_document_fragment(input: &str) -> MarkdownDocument {
        parse_document_inner(input)
    }

    fn parse_document_inner(input: &str) -> MarkdownDocument {
        let mut metadata = ParserMetadata::default();
        let lines: Vec<&str> = input.lines().collect();
        let body_lines = extract_footnote_definitions(&lines, &mut metadata);
        let body_lines = extract_link_definitions(&body_lines, &mut metadata);

        let link_defs: LinkDefinitions = metadata
            .references
            .iter()
            .map(|reference| {
                (
                    reference.label.clone(),
                    (reference.url.clone(), reference.title.clone()),
                )
            })
            .collect();

        MarkdownDocument {
            blocks: parse_blocks(&body_lines, &link_defs),
            footnotes: metadata.footnotes,
            references: metadata.references,
        }
    }

    #[derive(Default)]
    struct ParserMetadata {
        footnotes: Vec<FootnoteDefinition>,
        references: Vec<LinkReferenceDefinition>,
    }

    fn extract_footnote_definitions<'a>(
        lines: &[&'a str],
        metadata: &mut ParserMetadata,
    ) -> Vec<&'a str> {
        let mut body = Vec::with_capacity(lines.len());
        let mut i = 0;

        while i < lines.len() {
            let line = lines[i];

            // Skip lines inside fenced code blocks
            if let Some((marker, _)) = fence_start(line) {
                let start_idx = i;
                i += 1;
                while i < lines.len() && !is_fence_close(lines[i], marker) {
                    i += 1;
                }
                // Add all lines inside the fenced block to body (as code block content)
                for j in start_idx..i {
                    body.push(lines[j]);
                }
                if i < lines.len() {
                    body.push(lines[i]);
                    i += 1;
                }
                continue;
            }

            // Skip lines inside indented code blocks
            if is_indented_code_line(line) {
                while i < lines.len() && is_indented_code_line(lines[i]) {
                    body.push(lines[i]);
                    i += 1;
                }
                continue;
            }

            if let Some((id, first_line)) = parse_footnote_definition(line) {
                let mut content = String::from(first_line);
                i += 1;

                while i < lines.len() {
                    let next = lines[i];
                    if next.starts_with("    ") || next.starts_with('\t') {
                        if !content.is_empty() {
                            content.push('\n');
                        }
                        content.push_str(next.trim_start());
                        i += 1;
                        continue;
                    }

                    if next.trim().is_empty()
                        && i + 1 < lines.len()
                        && (lines[i + 1].starts_with("    ") || lines[i + 1].starts_with('\t'))
                    {
                        content.push_str("\n\n");
                        i += 1;
                        continue;
                    }

                    break;
                }

                metadata.footnotes.push(FootnoteDefinition {
                    id: id.to_string(),
                    content: content.trim().to_string(),
                });
                continue;
            }

            body.push(line);
            i += 1;
        }

        body
    }

    fn extract_link_definitions<'a>(
        lines: &[&'a str],
        metadata: &mut ParserMetadata,
    ) -> Vec<&'a str> {
        let mut body = Vec::with_capacity(lines.len());
        let mut i = 0;

        while i < lines.len() {
            let line = lines[i];

            // Skip lines inside fenced code blocks
            if let Some((marker, _)) = fence_start(line) {
                let start_idx = i;
                i += 1;
                while i < lines.len() && !is_fence_close(lines[i], marker) {
                    i += 1;
                }
                // Add all lines inside the fenced block to body (as code block content)
                for j in start_idx..i {
                    body.push(lines[j]);
                }
                if i < lines.len() {
                    body.push(lines[i]);
                    i += 1;
                }
                continue;
            }

            // Skip lines inside indented code blocks
            if is_indented_code_line(line) {
                while i < lines.len() && is_indented_code_line(lines[i]) {
                    body.push(lines[i]);
                    i += 1;
                }
                continue;
            }

            if let Some((label, url, title)) = parse_link_definition(line) {
                metadata
                    .references
                    .push(LinkReferenceDefinition { label, url, title });
            } else {
                body.push(line);
            }
            i += 1;
        }

        body
    }

    fn parse_footnote_definition(line: &str) -> Option<(&str, &str)> {
        let trimmed = line.trim_start();
        if !trimmed.starts_with("[^") {
            return None;
        }

        let closing = trimmed.find("]:")?;
        let id = &trimmed[2..closing];
        if id.is_empty() || id.chars().any(char::is_whitespace) {
            return None;
        }

        Some((id, trimmed[closing + 2..].trim_start()))
    }

    fn parse_link_definition(line: &str) -> Option<(String, String, Option<String>)> {
        let trimmed = line.trim_start();
        if !trimmed.starts_with('[') || trimmed.starts_with("[^") {
            return None;
        }

        let closing = trimmed.find("]:")?;
        let label = normalize_reference_label(&trimmed[1..closing]);
        if label.is_empty() {
            return None;
        }

        let (url, title) = parse_destination_and_title(trimmed[closing + 2..].trim_start())?;
        Some((label, url, title))
    }

    pub(crate) fn parse_blocks(lines: &[&str], link_defs: &LinkDefinitions) -> Vec<Block> {
        let mut blocks = Vec::new();
        let len = lines.len();
        let mut i = 0;

        while i < len {
            let line = lines[i];

            if line.trim().is_empty() {
                i += 1;
                continue;
            }

            if is_indented_code_line(line) {
                let mut code = String::new();
                while i < len {
                    if lines[i].trim().is_empty() {
                        code.push('\n');
                        i += 1;
                        continue;
                    }
                    if !is_indented_code_line(lines[i]) {
                        break;
                    }
                    code.push_str(strip_indented_code_prefix(lines[i]));
                    code.push('\n');
                    i += 1;
                }
                blocks.push(Block::CodeBlock(CodeBlock {
                    language: None,
                    code,
                }));
                continue;
            }

            if let Some((marker, lang)) = fence_start(line) {
                let mut code = String::new();
                i += 1;
                while i < len {
                    if is_fence_close(lines[i], marker) {
                        i += 1;
                        break;
                    }
                    code.push_str(lines[i]);
                    code.push('\n');
                    i += 1;
                }
                blocks.push(Block::CodeBlock(CodeBlock {
                    language: (!lang.is_empty()).then(|| lang.to_string()),
                    code,
                }));
                continue;
            }

            if let Some(level) = parse_setext_heading(lines, i) {
                blocks.push(Block::Heading(HeadingBlock {
                    level,
                    content: inline_content(lines[i].trim(), link_defs),
                    id: None,
                }));
                i += 2;
                continue;
            }

            if let Some((headers, alignments)) = parse_table_start(lines, i) {
                let mut rows: Vec<Vec<String>> = Vec::new();
                i += 2;

                while i < len {
                    let Some(cells) = common::split_table_row(lines[i]) else {
                        break;
                    };
                    if cells.len() != headers.len()
                        || common::parse_table_alignments(lines[i]).is_some()
                    {
                        break;
                    }
                    rows.push(cells.into_iter().map(str::to_string).collect());
                    i += 1;
                }

                blocks.push(Block::Table(TableBlock {
                    headers: headers
                        .into_iter()
                        .map(|header| inline_content(header, link_defs))
                        .collect(),
                    alignments,
                    rows: rows
                        .into_iter()
                        .map(|row| {
                            row.into_iter()
                                .map(|cell| inline_content(&cell, link_defs))
                                .collect::<Vec<_>>()
                        })
                        .collect::<Vec<_>>(),
                }));
                continue;
            }

            if let Some((level, text, id)) = parse_heading(line) {
                blocks.push(Block::Heading(HeadingBlock {
                    level,
                    content: inline_content(text, link_defs),
                    id: id.map(str::to_string),
                }));
                i += 1;
                continue;
            }

            if is_horizontal_rule(line) {
                blocks.push(Block::HorizontalRule);
                i += 1;
                continue;
            }

            if line.trim_start().starts_with('>') {
                let mut blockquote_lines = Vec::new();
                while i < len
                    && (lines[i].trim_start().starts_with('>') || lines[i].trim().is_empty())
                {
                    let current = lines[i];
                    if current.trim().is_empty() {
                        if i + 1 < len && lines[i + 1].trim_start().starts_with('>') {
                            blockquote_lines.push(String::new());
                            i += 1;
                            continue;
                        }
                        break;
                    }

                    let stripped = current.trim_start().strip_prefix('>').unwrap_or(current);
                    blockquote_lines.push(stripped.trim_start().to_string());
                    i += 1;
                }

                let inner_lines: Vec<&str> = blockquote_lines.iter().map(String::as_str).collect();
                blocks.push(Block::BlockQuote(parse_blocks(&inner_lines, link_defs)));
                continue;
            }

            if is_definition_list_start(lines, i) {
                let mut items = Vec::new();
                while i < len {
                    if !is_definition_list_start(lines, i) {
                        break;
                    }

                    let term = inline_content(lines[i].trim(), link_defs);
                    i += 1;
                    let mut definitions = Vec::new();
                    while i < len && lines[i].trim_start().starts_with(": ") {
                        definitions.push(inline_content(
                            lines[i].trim_start()[2..].trim_start(),
                            link_defs,
                        ));
                        i += 1;
                    }
                    items.push(DefinitionItem { term, definitions });
                }
                blocks.push(Block::DefinitionList(items));
                continue;
            }

            if is_task_list_item(line) {
                let mut items = Vec::new();
                while i < len && is_task_list_item(lines[i]) {
                    let (checked, text) = parse_task_list_item(lines[i]).expect("task list item");
                    items.push(TaskItem {
                        checked,
                        content: inline_content(text, link_defs),
                    });
                    i += 1;
                }
                blocks.push(Block::TaskList(items));
                continue;
            }

            if is_unordered_list_item(line) {
                let mut items = Vec::new();
                let list_indent = leading_indent_width(line);
                let mut current_text = String::new();
                while i < len {
                    if lines[i].trim().is_empty() {
                        if is_blank_line_followed_by_list_continuation(lines, i) {
                            i += 1;
                            continue;
                        }
                        if i + 1 < len && is_unordered_list_item(lines[i + 1]) {
                            // Blank line between list items — flush current and continue
                            if !current_text.is_empty() {
                                items.push(ListItem {
                                    content: inline_content(&current_text, link_defs),
                                });
                                current_text.clear();
                            }
                            i += 1;
                            continue;
                        }
                        break;
                    }
                    if is_unordered_list_item(lines[i]) {
                        let indent = leading_indent_width(lines[i]);
                        if indent != list_indent {
                            if !current_text.is_empty() {
                                items.push(ListItem {
                                    content: inline_content(&current_text, link_defs),
                                });
                                current_text.clear();
                            }
                            break;
                        }
                        if !current_text.is_empty() {
                            items.push(ListItem {
                                content: inline_content(&current_text, link_defs),
                            });
                            current_text.clear();
                        }
                        let text = lines[i]
                            .trim_start()
                            .strip_prefix('-')
                            .or_else(|| lines[i].trim_start().strip_prefix('*'))
                            .unwrap_or(lines[i])
                            .trim_start();
                        current_text.push_str(text);
                    } else if is_indented_sub_item(lines[i])
                        && leading_indent_width(lines[i]) > list_indent
                    {
                        if !current_text.is_empty() {
                            items.push(ListItem {
                                content: inline_content(&current_text, link_defs),
                            });
                            current_text.clear();
                        }
                        break;
                    } else if is_continuation_line(lines[i]) || is_indented_sub_item(lines[i]) {
                        if !current_text.is_empty() {
                            current_text.push(' ');
                        }
                        current_text.push_str(lines[i].trim());
                    } else {
                        break;
                    }
                    i += 1;
                }
                if !current_text.is_empty() {
                    items.push(ListItem {
                        content: inline_content(&current_text, link_defs),
                    });
                }
                blocks.push(Block::UnorderedList(UnorderedListBlock {
                    indent: list_indent,
                    items,
                }));
                continue;
            }

            if is_ordered_list_item(line) {
                let mut items = Vec::new();
                let list_indent = leading_indent_width(line);
                let mut start = None;
                let mut current_text = String::new();
                while i < len {
                    if lines[i].trim().is_empty() {
                        if is_blank_line_followed_by_list_continuation(lines, i) {
                            i += 1;
                            continue;
                        }
                        // Blank line between ordered list items
                        if i + 1 < len && is_ordered_list_item(lines[i + 1]) {
                            if !current_text.is_empty() {
                                items.push(ListItem {
                                    content: inline_content(&current_text, link_defs),
                                });
                                current_text.clear();
                            }
                            i += 1;
                            continue;
                        }
                        break;
                    }
                    if is_ordered_list_item(lines[i]) {
                        let indent = leading_indent_width(lines[i]);
                        if indent != list_indent {
                            if !current_text.is_empty() {
                                items.push(ListItem {
                                    content: inline_content(&current_text, link_defs),
                                });
                                current_text.clear();
                            }
                            break;
                        }
                        if start.is_none() {
                            start = ordered_list_start(lines[i]);
                        }
                        // New list item — flush previous
                        if !current_text.is_empty() {
                            items.push(ListItem {
                                content: inline_content(&current_text, link_defs),
                            });
                            current_text.clear();
                        }
                        current_text.push_str(strip_ordered_prefix(lines[i]));
                    } else if is_indented_sub_item(lines[i])
                        && leading_indent_width(lines[i]) > list_indent
                    {
                        if !current_text.is_empty() {
                            items.push(ListItem {
                                content: inline_content(&current_text, link_defs),
                            });
                            current_text.clear();
                        }
                        break;
                    } else if is_continuation_line(lines[i])
                        || is_lazy_list_continuation_line(lines, i)
                    {
                        if !current_text.is_empty() {
                            current_text.push(' ');
                        }
                        current_text.push_str(lines[i].trim());
                    } else {
                        break;
                    }
                    i += 1;
                }
                if !current_text.is_empty() {
                    items.push(ListItem {
                        content: inline_content(&current_text, link_defs),
                    });
                }
                blocks.push(Block::OrderedList(OrderedListBlock {
                    start: start.unwrap_or(1),
                    indent: list_indent,
                    items,
                }));
                continue;
            }

            let mut paragraph = Vec::new();
            while i < len && !lines[i].trim().is_empty() && !is_special_block_start(lines, i) {
                paragraph.push(ParagraphLine {
                    content: inline_content(trim_hard_break_marker(lines[i]), link_defs),
                    hard_break: has_hard_break(lines[i]),
                });
                i += 1;
            }

            if !paragraph.is_empty() {
                let content = inline_content(
                    &super::markdownish::paragraph_source_from_lines(&paragraph),
                    link_defs,
                );
                blocks.push(Block::Paragraph(ParagraphBlock {
                    lines: paragraph,
                    content,
                }));
            }
        }

        blocks
    }

    fn inline_content(source: &str, link_defs: &LinkDefinitions) -> InlineContent {
        InlineContent {
            source: source.to_string(),
            nodes: parse_inline_nodes(source, link_defs),
        }
    }

    fn leading_indent_width(line: &str) -> usize {
        line.chars()
            .take_while(|ch| ch.is_ascii_whitespace())
            .count()
    }
}
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
impl DocumentCache {
    /// Parse a full markdown document, using the cache when possible.
    ///
    /// Full documents are cached separately from fragments. Footnotes are
    /// only rendered for full documents.
    pub fn parse_document(&mut self, input: &str) -> MarkdownDocument {
        if let Some(document) = self.full_documents.get(input).cloned() {
            touch_cache_order(&mut self.full_order, input);
            return document;
        }

        let document = parse::parse_document(input);
        insert_cached_document(
            &mut self.full_documents,
            &mut self.full_order,
            &mut self.full_document_bytes,
            input,
            &document,
        );
        document
    }

    /// Parse a markdown fragment, using the cache when possible.
    ///
    /// Fragments are cached separately from full documents and do not
    /// render footnote sections.
    pub fn parse_document_fragment(&mut self, input: &str) -> MarkdownDocument {
        if let Some(document) = self.fragment_documents.get(input).cloned() {
            touch_cache_order(&mut self.fragment_order, input);
            return document;
        }

        let document = parse::parse_document_fragment(input);
        insert_cached_document(
            &mut self.fragment_documents,
            &mut self.fragment_order,
            &mut self.fragment_document_bytes,
            input,
            &document,
        );
        document
    }

    /// Clear all cached documents and fragments, freeing memory.
    pub fn clear(&mut self) {
        self.full_documents.clear();
        self.fragment_documents.clear();
        self.full_order.clear();
        self.fragment_order.clear();
        self.full_document_bytes = 0;
        self.fragment_document_bytes = 0;
    }
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
/// Parse markdown input into a [`MarkdownDocument`] with block-level structure,
/// inline content, footnotes, and link reference definitions.
pub fn parse_document(input: &str) -> MarkdownDocument {
    parse::parse_document(input)
}
/// Parse a markdown fragment into a [`MarkdownDocument`] without footnote extraction.
pub fn parse_document_fragment(input: &str) -> MarkdownDocument {
    parse::parse_document_fragment(input)
}
/// Parse a full document with caching via [`DocumentCache`].
pub fn parse_document_cached(cache: &mut DocumentCache, input: &str) -> MarkdownDocument {
    cache.parse_document(input)
}
/// Parse a fragment with caching via [`DocumentCache`].
pub fn parse_document_fragment_cached(cache: &mut DocumentCache, input: &str) -> MarkdownDocument {
    cache.parse_document_fragment(input)
}
