use std::collections::HashMap;
use crate::parse_util::{
    find_code_span_end, find_sequence, is_markdown_escapable, looks_like_email,
    looks_like_html_tag, looks_like_url, normalize_reference_label, parse_angle_bracket_content,
    parse_autolink, parse_bracket_content, parse_emoji_shortcode, parse_footnote_reference,
    parse_html_entity, parse_link_destination, starts_with_chars,
};
/// A map of link reference labels to their URL and optional title.
///
/// Used by [`parse_inline_nodes`] to resolve reference-style links like `[text][label]`.
pub type LinkDefinitions = HashMap<String, (String, Option<String>)>;
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
/// Parse inline markdown content into a tree of [`InlineNode`] values.
///
/// Handles emphasis, strong, strikethrough, code spans, links, images,
/// footnote references, autolinks, HTML entities, emoji shortcodes, and
/// reference-style links resolved through `link_defs`.
pub fn parse_inline_nodes(input: &str, link_defs: &LinkDefinitions) -> Vec<InlineNode> {
    let chars: Vec<char> = input.chars().collect();
    let mut out = Vec::new();
    let mut i = 0;

    while i < chars.len() {
        if chars[i] == '&' {
            if let Some((entity, end)) = parse_html_entity(&chars, i) {
                out.push(InlineNode::Entity(entity));
                i = end;
                continue;
            }
        }

        if chars[i] == '\\' && i + 1 < chars.len() && is_markdown_escapable(chars[i + 1]) {
            push_text(&mut out, chars[i + 1].to_string());
            i += 2;
            continue;
        }

        if chars[i] == '`' {
            if let Some(end) = find_code_span_end(&chars, i) {
                // Count opening backticks to determine delimiter length
                let mut open_len = 0;
                let mut j = i;
                while j < chars.len() && chars[j] == '`' {
                    open_len += 1;
                    j += 1;
                }
                // end is exclusive (after all closing backticks), so subtract open_len to exclude closing delimiters
                let code: String = chars[i + open_len..end - open_len].iter().collect();
                out.push(InlineNode::Code(code));
                i = end + 1;
                continue;
            }
        }

        if chars[i] == '<' {
            if let Some((inner, end)) = parse_angle_bracket_content(&chars, i) {
                if looks_like_url(&inner) {
                    out.push(InlineNode::Link {
                        children: vec![InlineNode::Text(inner.clone())],
                        url: inner,
                        title: None,
                    });
                    i = end;
                    continue;
                }
                if looks_like_email(&inner) {
                    out.push(InlineNode::Link {
                        children: vec![InlineNode::Text(inner.clone())],
                        url: format!("mailto:{inner}"),
                        title: None,
                    });
                    i = end;
                    continue;
                }
                if looks_like_html_tag(&inner) {
                    out.push(InlineNode::RawHtml(format!("<{inner}>")));
                    i = end;
                    continue;
                }
            }
        }

        if chars[i] == '!' && i + 1 < chars.len() && chars[i + 1] == '[' {
            if let Some((alt, url, title, end)) = parse_image(&chars, i, link_defs) {
                out.push(InlineNode::Image { alt, url, title });
                i = end;
                continue;
            }
        }

        if chars[i] == '[' {
            if let Some((id, end)) = parse_footnote_reference(&chars, i) {
                out.push(InlineNode::FootnoteReference(id));
                i = end;
                continue;
            }

            if let Some((text, url, title, end)) = parse_link(&chars, i, link_defs) {
                out.push(InlineNode::Link {
                    children: parse_inline_nodes(&text, link_defs),
                    url,
                    title,
                });
                i = end;
                continue;
            }
        }

        if starts_with_chars(&chars, i, "http://") || starts_with_chars(&chars, i, "https://") {
            let (url, trimmed_end, end) = parse_autolink(&chars, i);
            out.push(InlineNode::Link {
                children: vec![InlineNode::Text(url.clone())],
                url,
                title: None,
            });
            if trimmed_end < end {
                push_text(&mut out, chars[trimmed_end..end].iter().collect::<String>());
            }
            i = end;
            continue;
        }

        if starts_with_chars(&chars, i, "***") || starts_with_chars(&chars, i, "___") {
            let pattern = if chars[i] == '*' { "***" } else { "___" };
            if let Some(end) = find_sequence(&chars, pattern, i + 3) {
                let inner: String = chars[i + 3..end].iter().collect();
                out.push(InlineNode::Emphasis(vec![InlineNode::Strong(
                    parse_inline_nodes(&inner, link_defs),
                )]));
                i = end + 3;
                continue;
            }
        }

        if starts_with_chars(&chars, i, "~~") {
            if let Some(end) = find_sequence(&chars, "~~", i + 2) {
                let inner: String = chars[i + 2..end].iter().collect();
                out.push(InlineNode::Strikethrough(parse_inline_nodes(
                    &inner, link_defs,
                )));
                i = end + 2;
                continue;
            }
        }

        if starts_with_chars(&chars, i, "==") {
            if let Some(end) = find_sequence(&chars, "==", i + 2) {
                let inner: String = chars[i + 2..end].iter().collect();
                out.push(InlineNode::Mark(parse_inline_nodes(&inner, link_defs)));
                i = end + 2;
                continue;
            }
        }

        if starts_with_chars(&chars, i, "**")
            || (starts_with_chars(&chars, i, "__") && can_open_underscore_marker(&chars, i, 2))
        {
            let pattern = if chars[i] == '*' { "**" } else { "__" };
            if let Some(end) = find_closing_sequence(&chars, pattern, i + 2) {
                let inner: String = chars[i + 2..end].iter().collect();
                out.push(InlineNode::Strong(parse_inline_nodes(&inner, link_defs)));
                i = end + 2;
                continue;
            }
        }

        if chars[i] == '*' || (chars[i] == '_' && can_open_underscore_marker(&chars, i, 1)) {
            if let Some(end) = find_single_marker(&chars, chars[i], i + 1) {
                let inner: String = chars[i + 1..end].iter().collect();
                out.push(InlineNode::Emphasis(parse_inline_nodes(&inner, link_defs)));
                i = end + 1;
                continue;
            }
        }

        if chars[i] == '~' && (i + 1 >= chars.len() || chars[i + 1] != '~') {
            if let Some(end) = find_single_marker(&chars, '~', i + 1) {
                let inner: String = chars[i + 1..end].iter().collect();
                if !inner.chars().any(char::is_whitespace) {
                    out.push(InlineNode::Subscript(parse_inline_nodes(&inner, link_defs)));
                    i = end + 1;
                    continue;
                }
            }
        }

        if chars[i] == '^' {
            if let Some(end) = find_single_marker(&chars, '^', i + 1) {
                let inner: String = chars[i + 1..end].iter().collect();
                if !inner.chars().any(char::is_whitespace) {
                    out.push(InlineNode::Superscript(parse_inline_nodes(
                        &inner, link_defs,
                    )));
                    i = end + 1;
                    continue;
                }
            }
        }

        if chars[i] == ':' {
            if let Some((emoji, end)) = parse_emoji_shortcode(&chars, i) {
                push_text(&mut out, emoji.to_string());
                i = end;
                continue;
            }
        }

        push_text(&mut out, chars[i].to_string());
        i += 1;
    }

    out
}
fn push_text(out: &mut Vec<InlineNode>, text: String) {
    if text.is_empty() {
        return;
    }

    if let Some(InlineNode::Text(existing)) = out.last_mut() {
        existing.push_str(&text);
    } else {
        out.push(InlineNode::Text(text));
    }
}
fn find_single_marker(chars: &[char], target: char, start: usize) -> Option<usize> {
    for index in start..chars.len() {
        if chars[index] == target {
            if index + 1 < chars.len() && chars[index + 1] == target {
                continue;
            }
            if target == '_' && !can_close_underscore_marker(chars, index, 1) {
                continue;
            }
            if index > start {
                return Some(index);
            }
        }
    }
    None
}
fn find_closing_sequence(chars: &[char], pattern: &str, start: usize) -> Option<usize> {
    let pattern_chars: Vec<char> = pattern.chars().collect();
    if pattern_chars.is_empty() || start >= chars.len() {
        return None;
    }

    (start..=chars.len().saturating_sub(pattern_chars.len())).find(|&index| {
        chars[index..index + pattern_chars.len()]
            .iter()
            .zip(pattern_chars.iter())
            .all(|(left, right)| left == right)
            && (pattern != "__" || can_close_underscore_marker(chars, index, 2))
    })
}
fn is_word_char(ch: Option<char>) -> bool {
    ch.is_some_and(char::is_alphanumeric)
}
fn can_open_underscore_marker(chars: &[char], index: usize, len: usize) -> bool {
    let prev = index.checked_sub(1).and_then(|pos| chars.get(pos)).copied();
    let next = chars.get(index + len).copied();
    !is_word_char(prev) || !is_word_char(next)
}
fn can_close_underscore_marker(chars: &[char], index: usize, len: usize) -> bool {
    let prev = index.checked_sub(1).and_then(|pos| chars.get(pos)).copied();
    let next = chars.get(index + len).copied();
    !is_word_char(prev) || !is_word_char(next)
}
fn parse_reference_lookup(
    chars: &[char],
    start: usize,
    fallback: &str,
    link_defs: &LinkDefinitions,
) -> Option<(String, Option<String>, usize)> {
    if chars.get(start) == Some(&'[') {
        let (label, end) = parse_bracket_content(chars, start)?;
        let key = if label.trim().is_empty() {
            normalize_reference_label(fallback)
        } else {
            normalize_reference_label(&label)
        };
        let (url, title) = link_defs.get(&key)?.clone();
        return Some((url, title, end));
    }

    let key = normalize_reference_label(fallback);
    let (url, title) = link_defs.get(&key)?.clone();
    Some((url, title, start))
}
fn parse_link(
    chars: &[char],
    start: usize,
    link_defs: &LinkDefinitions,
) -> Option<(String, String, Option<String>, usize)> {
    let (text, after_text) = parse_bracket_content(chars, start)?;
    let (url, title, end) = if chars.get(after_text) == Some(&'(') {
        parse_link_destination(chars, after_text)?
    } else {
        parse_reference_lookup(chars, after_text, &text, link_defs)?
    };
    Some((text, url, title, end))
}
fn parse_image(
    chars: &[char],
    start: usize,
    link_defs: &LinkDefinitions,
) -> Option<(String, String, Option<String>, usize)> {
    if chars.get(start) != Some(&'!') || chars.get(start + 1) != Some(&'[') {
        return None;
    }

    let (alt, after_alt) = parse_bracket_content(chars, start + 1)?;
    let (url, title, end) = if chars.get(after_alt) == Some(&'(') {
        parse_link_destination(chars, after_alt)?
    } else {
        parse_reference_lookup(chars, after_alt, &alt, link_defs)?
    };
    Some((alt, url, title, end))
}
