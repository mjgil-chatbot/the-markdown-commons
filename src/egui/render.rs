use crate::document::Block;
use crate::document::InlineContent;
use crate::inline::InlineNode;
use egui_extras::syntax_highlighting::{self, CodeTheme};
trait TableCellContent {
    fn source_text(&self) -> &str;

    fn inline_nodes(&self) -> Option<&[crate::inline::InlineNode]> {
        None
    }
}
impl TableCellContent for String {
    fn source_text(&self) -> &str {
        self
    }
}
impl TableCellContent for InlineContent {
    fn source_text(&self) -> &str {
        &self.source
    }

    fn inline_nodes(&self) -> Option<&[crate::inline::InlineNode]> {
        Some(&self.nodes)
    }
}
trait FromInlineContentCell: Sized {
    fn from_inline_content(content: &InlineContent) -> Self;
}
impl FromInlineContentCell for String {
    fn from_inline_content(content: &InlineContent) -> Self {
        content.source.clone()
    }
}
impl FromInlineContentCell for InlineContent {
    fn from_inline_content(content: &InlineContent) -> Self {
        content.clone()
    }
}
/// Options for markdown rendering.
#[derive(Default, Clone, Debug)]
pub struct MarkdownRenderOptions<'a> {
    /// Language to use for fenced code blocks that have no language tag.
    /// For example, `Some("sh")` treats bare ``` blocks as shell.
    pub default_code_language: Option<&'a str>,
    /// Map specific language tags to a different language for highlighting.
    /// For example, `&[("text", "sh"), ("txt", "sh")]` treats ````text`
    /// blocks as shell. Entries are checked before syntect receives the tag.
    pub code_language_overrides: &'a [(&'a str, &'a str)],
}
fn collect_ordered_list_continuation(
    blocks: &[Block],
    start_index: usize,
    current_indent: usize,
    expected_start: usize,
) -> (&[Block], Option<usize>, usize) {
    let mut index = start_index;

    while index < blocks.len() {
        match &blocks[index] {
            Block::OrderedList(list)
                if list.indent == current_indent && list.start == expected_start =>
            {
                return (&blocks[start_index..index], Some(index), index);
            }
            Block::OrderedList(list) if list.indent <= current_indent => break,
            Block::Paragraph(paragraph) if is_section_label_paragraph(&paragraph.content.nodes) => {
                break;
            }
            Block::Heading(_) | Block::HorizontalRule => break,
            _ => index += 1,
        }
    }

    (&blocks[start_index..index], None, index)
}
fn collect_unordered_list_continuation(
    blocks: &[Block],
    start_index: usize,
    current_indent: usize,
) -> (&[Block], Option<usize>, usize) {
    let mut index = start_index;

    while index < blocks.len() {
        match &blocks[index] {
            Block::UnorderedList(list) if list.indent == current_indent => {
                return (&blocks[start_index..index], Some(index), index);
            }
            Block::UnorderedList(list) if list.indent > current_indent => index += 1,
            Block::OrderedList(list) if list.indent > current_indent => index += 1,
            Block::Paragraph(_) | Block::Heading(_) | Block::HorizontalRule => break,
            Block::UnorderedList(_) | Block::OrderedList(_) => break,
            _ => break,
        }
    }

    (&blocks[start_index..index], None, index)
}
fn is_section_label_paragraph(nodes: &[InlineNode]) -> bool {
    matches!(nodes, [InlineNode::Strong(children)] if matches!(children.as_slice(), [InlineNode::Text(_)]))
}
