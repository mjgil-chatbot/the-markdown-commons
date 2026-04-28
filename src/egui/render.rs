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
