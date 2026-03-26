use std::collections::HashMap;
/// Inline-level nodes parsed from markdown content.
///
/// `InlineNode` represents text-level formatting: emphasis, strong, links,
/// images, code spans, entities, etc. It forms a tree structure where
/// container nodes like `Emphasis` hold child nodes.
///
/// # Example
///
/// ```
/// use tmc::{parse_inline_nodes, InlineNode, LinkDefinitions};
///
/// let nodes = parse_inline_nodes("Hello **world**", &LinkDefinitions::default());
/// // nodes[0] is Text("Hello ")
/// // nodes[1] is Strong(vec![Text("world")])
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InlineNode {
    /// Plain text content.
    Text(String),
    /// An HTML entity (e.g., `&lt;`, `&amp;`, `&#124;`).
    Entity(String),
    /// An inline code span.
    Code(String),
    /// Emphasized text (`*text*` or `_text_`).
    Emphasis(Vec<InlineNode>),
    /// Strong text (`**text**` or `__text__`).
    Strong(Vec<InlineNode>),
    /// Strikethrough text (`~~text~~`).
    Strikethrough(Vec<InlineNode>),
    /// Highlighted/mark text (`==text==`).
    Mark(Vec<InlineNode>),
    /// Subscript text (`~text~`).
    Subscript(Vec<InlineNode>),
    /// Superscript text (`^text^`).
    Superscript(Vec<InlineNode>),
    /// A hyperlink.
    Link {
        /// The inner content of the link.
        children: Vec<InlineNode>,
        /// The destination URL.
        url: String,
        /// An optional title attribute.
        title: Option<String>,
    },
    /// An image.
    Image {
        /// The alt text.
        alt: String,
        /// The image URL.
        url: String,
        /// An optional title attribute.
        title: Option<String>,
    },
    /// A footnote reference.
    FootnoteReference(String),
    /// Raw HTML content (when `allow_raw_html` is enabled).
    RawHtml(String),
}
