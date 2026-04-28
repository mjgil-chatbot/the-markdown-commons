use super::ParserState;
use crate::inline::InlineNode;

pub(super) fn render_inline_nodes(nodes: &[InlineNode], state: &mut ParserState) -> String {
    render_inline_nodes_html(nodes, state)
}

fn render_inline_nodes_html(nodes: &[InlineNode], state: &mut ParserState) -> String {
    let mut out = String::new();

    for node in nodes {
        match node {
            InlineNode::Text(text) => out.push_str(&super::escape_html(text)),
            InlineNode::Entity(entity) => out.push_str(entity),
            InlineNode::Code(code) => {
                out.push_str(&format!("<code>{}</code>", super::escape_html(code)));
            }
            InlineNode::Emphasis(children) => {
                out.push_str(&format!(
                    "<em>{}</em>",
                    render_inline_nodes_html(children, state)
                ));
            }
            InlineNode::Strong(children) => {
                out.push_str(&format!(
                    "<strong>{}</strong>",
                    render_inline_nodes_html(children, state)
                ));
            }
            InlineNode::Strikethrough(children) => {
                out.push_str(&format!(
                    "<del>{}</del>",
                    render_inline_nodes_html(children, state)
                ));
            }
            InlineNode::Mark(children) => {
                out.push_str(&format!(
                    "<mark>{}</mark>",
                    render_inline_nodes_html(children, state)
                ));
            }
            InlineNode::Subscript(children) => {
                out.push_str(&format!(
                    "<sub>{}</sub>",
                    render_inline_nodes_html(children, state)
                ));
            }
            InlineNode::Superscript(children) => {
                out.push_str(&format!(
                    "<sup>{}</sup>",
                    render_inline_nodes_html(children, state)
                ));
            }
            InlineNode::Link {
                children,
                url,
                title,
            } => {
                out.push_str(&super::render_link(
                    url,
                    title.as_deref(),
                    &render_inline_nodes_html(children, state),
                    &state.options,
                ));
            }
            InlineNode::Image { alt, url, title } => {
                out.push_str(&super::render_image(url, title.as_deref(), alt));
            }
            InlineNode::FootnoteReference(id) => {
                let number = state.footnote_number(id);
                out.push_str(&format!(
                    r##"<sup id="fnref:{id}"><a href="#fn:{id}" class="footnote-ref">{number}</a></sup>"##,
                    id = super::escape_html(id),
                ));
            }
            InlineNode::RawHtml(raw) => {
                if state.options.allow_raw_html {
                    out.push_str(raw);
                } else {
                    out.push_str(&super::escape_html(raw));
                }
            }
        }
    }

    out
}
