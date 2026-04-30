//! HTML rendering from the shared markdown document model.

use crate::document::{
    parse_document, parse_document_cached, parse_document_fragment, parse_document_fragment_cached,
    DocumentCache, MarkdownDocument,
};
use std::collections::HashMap;

mod block;
mod inline;

use block::render_document_html;

/// Options for HTML rendering of markdown content.
///
/// # Example
///
/// ```
/// use tmc::{parse_markdown_with_options, HtmlOptions};
///
/// // Default: raw HTML allowed, links open in new tab
/// let html = parse_markdown_with_options("<em>raw</em>", HtmlOptions::default());
/// assert!(html.contains("<em>")); // raw HTML passes through when allow_raw_html is true
///
/// // Safe mode: disable raw HTML and blank targets
/// let safe_html = parse_markdown_with_options(
///     "<em>raw</em> [link](https://example.com)",
///     HtmlOptions {
///         link_target_blank: false,
///         allow_raw_html: false,
///     },
/// );
/// assert!(safe_html.contains("&lt;em&gt;")); // raw HTML is escaped when allow_raw_html is false
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HtmlOptions {
    /// When `true`, links render with `target="_blank"` for opening in a new tab.
    ///
    /// Defaults to `true`.
    pub link_target_blank: bool,
    /// When `true`, raw HTML tags in markdown are passed through unescaped.
    /// When `false`, raw HTML is escaped for safety.
    ///
    /// Defaults to `true` for backwards compatibility, but consider disabling
    /// (`false`) when rendering untrusted markdown to prevent XSS attacks.
    pub allow_raw_html: bool,
}

impl Default for HtmlOptions {
    fn default() -> Self {
        Self {
            link_target_blank: true,
            allow_raw_html: true,
        }
    }
}

struct ParserState {
    options: HtmlOptions,
    footnote_defs: HashMap<String, String>,
    footnote_order: Vec<String>,
}

impl ParserState {
    fn from_document(document: &MarkdownDocument, options: HtmlOptions) -> Self {
        let footnote_defs = document
            .footnotes
            .iter()
            .map(|footnote| (footnote.id.clone(), footnote.content.clone()))
            .collect();

        Self {
            options,
            footnote_defs,
            footnote_order: Vec::new(),
        }
    }

    fn footnote_number(&mut self, id: &str) -> usize {
        if let Some(index) = self
            .footnote_order
            .iter()
            .position(|existing| existing == id)
        {
            index + 1
        } else {
            self.footnote_order.push(id.to_string());
            self.footnote_order.len()
        }
    }
}

/// Parse a full markdown document into HTML.
pub fn parse_markdown(input: &str) -> String {
    parse_markdown_with_options(input, HtmlOptions::default())
}

/// Parse a full markdown document into HTML using a reusable document cache.
pub fn parse_markdown_cached(cache: &mut DocumentCache, input: &str) -> String {
    parse_markdown_with_options_cached(cache, input, HtmlOptions::default())
}

/// Parse a full markdown document into HTML with explicit options.
pub fn parse_markdown_with_options(input: &str, options: HtmlOptions) -> String {
    let document = parse_document(input);
    render_document_html(&document, options, true)
}

/// Parse a full markdown document into HTML with explicit options and a reusable document cache.
pub fn parse_markdown_with_options_cached(
    cache: &mut DocumentCache,
    input: &str,
    options: HtmlOptions,
) -> String {
    let document = parse_document_cached(cache, input);
    render_document_html(&document, options, true)
}

/// Parse a markdown fragment into HTML without appending a footnote section.
pub fn parse_markdown_fragment(input: &str) -> String {
    parse_markdown_fragment_with_options(input, HtmlOptions::default())
}

/// Parse a markdown fragment into HTML using a reusable document cache.
pub fn parse_markdown_fragment_cached(cache: &mut DocumentCache, input: &str) -> String {
    parse_markdown_fragment_with_options_cached(cache, input, HtmlOptions::default())
}

/// Parse a markdown fragment into HTML without appending a footnote section.
pub fn parse_markdown_fragment_with_options(input: &str, options: HtmlOptions) -> String {
    let document = parse_document_fragment(input);
    render_document_html(&document, options, false)
}

/// Parse a markdown fragment into HTML with explicit options and a reusable document cache.
pub fn parse_markdown_fragment_with_options_cached(
    cache: &mut DocumentCache,
    input: &str,
    options: HtmlOptions,
) -> String {
    let document = parse_document_fragment_cached(cache, input);
    render_document_html(&document, options, false)
}

fn render_link(url: &str, title: Option<&str>, body: &str, options: &HtmlOptions) -> String {
    let title_attr = title
        .map(|value| format!(r#" title="{}""#, escape_html(value)))
        .unwrap_or_default();
    let target_attr = if options.link_target_blank {
        r#" target="_blank""#
    } else {
        ""
    };
    format!(
        r#"<a href="{href}"{title_attr}{target_attr}>{body}</a>"#,
        href = escape_html(url)
    )
}

fn render_image(url: &str, title: Option<&str>, alt: &str) -> String {
    let title_attr = title
        .map(|value| format!(r#" title="{}""#, escape_html(value)))
        .unwrap_or_default();
    format!(
        r#"<img src="{src}" alt="{alt}"{title_attr}>"#,
        src = escape_html(url),
        alt = escape_html(alt)
    )
}

fn escape_html(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

#[cfg(test)]
mod tests;
