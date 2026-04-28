use super::{
    escape_html, parse_markdown_fragment_with_options, HtmlOptions, MarkdownDocument, ParserState,
};
use crate::common::Alignment;
use crate::document::{
    Block, CodeBlock, DefinitionItem, HeadingBlock, OrderedListBlock, ParagraphBlock, TableBlock,
    TaskItem, UnorderedListBlock,
};
use crate::html::inline::render_inline_nodes;
use crate::inline::InlineNode;

pub(super) fn render_document_html(
    document: &MarkdownDocument,
    options: HtmlOptions,
    include_footnotes: bool,
) -> String {
    let mut state = ParserState::from_document(document, options);
    let mut html = render_blocks_html(&document.blocks, &mut state);
    if include_footnotes {
        html.push_str(&render_footnotes(&state));
    }
    html
}

fn render_blocks_html(blocks: &[Block], state: &mut ParserState) -> String {
    let mut html = String::new();
    let mut index = 0;

    while index < blocks.len() {
        if let Block::UnorderedList(list) = &blocks[index] {
            let (rendered, next_index) = render_unordered_list_sequence(blocks, index, list, state);
            html.push_str(&rendered);
            index = next_index;
            continue;
        }
        if let Block::OrderedList(list) = &blocks[index] {
            let (rendered, next_index) = render_ordered_list_sequence(blocks, index, list, state);
            html.push_str(&rendered);
            index = next_index;
            continue;
        }

        html.push_str(&render_single_block_html(&blocks[index], state));
        index += 1;
    }

    html
}

fn render_single_block_html(block: &Block, state: &mut ParserState) -> String {
    match block {
        Block::Paragraph(paragraph) => render_paragraph_html(paragraph, state),
        Block::Heading(heading) => render_heading_html(heading, state),
        Block::CodeBlock(code) => render_code_block_html(code),
        Block::Table(table) => render_table_html(table, state),
        Block::BlockQuote(inner) => {
            format!(
                "<blockquote>{}</blockquote>\n",
                render_blocks_html(inner, state)
            )
        }
        Block::DefinitionList(items) => render_definition_list_html(items, state),
        Block::TaskList(items) => render_task_list_html(items, state),
        Block::UnorderedList(list) => render_unordered_list_html(list, state),
        Block::OrderedList(list) => render_ordered_list_html(list, state),
        Block::HorizontalRule => String::from("<hr>\n"),
    }
}

fn render_unordered_list_sequence(
    blocks: &[Block],
    start_index: usize,
    first_list: &UnorderedListBlock,
    state: &mut ParserState,
) -> (String, usize) {
    let mut html = String::from("<ul>\n");
    let mut current_list = first_list;
    let mut index = start_index;

    loop {
        let (continuation_blocks, next_unordered_index, next_index_after_continuation) =
            collect_unordered_list_continuation(blocks, index + 1, current_list.indent);
        let (last_item, leading_items) = current_list
            .items
            .split_last()
            .expect("unordered lists should contain at least one item");

        for item in leading_items {
            html.push_str("<li>");
            html.push_str(&render_inline_nodes(&item.content.nodes, state));
            html.push_str("</li>\n");
        }

        html.push_str("<li>");
        html.push_str(&render_inline_nodes(&last_item.content.nodes, state));

        for block in continuation_blocks {
            html.push('\n');
            html.push_str(&render_single_block_html(block, state));
        }

        if let Some(next_unordered_index) = next_unordered_index {
            html.push_str("</li>\n");
            index = next_unordered_index;
            let Block::UnorderedList(next_list) = &blocks[index] else {
                unreachable!("unordered list continuation index must point to unordered list");
            };
            current_list = next_list;
            continue;
        }

        html.push_str("</li>\n");
        html.push_str("</ul>\n");
        return (html, next_index_after_continuation);
    }
}

fn render_ordered_list_sequence(
    blocks: &[Block],
    start_index: usize,
    first_list: &OrderedListBlock,
    state: &mut ParserState,
) -> (String, usize) {
    let mut html = open_ordered_list_tag(first_list.start);
    let mut current_list = first_list;
    let mut index = start_index;

    loop {
        let expected_next_start = current_list.start + current_list.items.len();
        let (continuation_blocks, next_ordered_index, next_index_after_continuation) =
            collect_ordered_list_continuation(
                blocks,
                index + 1,
                current_list.indent,
                expected_next_start,
            );
        let (last_item, leading_items) = current_list
            .items
            .split_last()
            .expect("ordered lists should contain at least one item");

        for item in leading_items {
            html.push_str("<li>");
            html.push_str(&render_inline_nodes(&item.content.nodes, state));
            html.push_str("</li>\n");
        }

        html.push_str("<li>");
        html.push_str(&render_inline_nodes(&last_item.content.nodes, state));

        for block in continuation_blocks {
            html.push('\n');
            html.push_str(&render_single_block_html(block, state));
        }

        if let Some(next_ordered_index) = next_ordered_index {
            html.push_str("</li>\n");
            index = next_ordered_index;
            let Block::OrderedList(next_list) = &blocks[index] else {
                unreachable!("ordered list continuation index must point to ordered list");
            };
            current_list = next_list;
            continue;
        }

        html.push_str("</li>\n");

        html.push_str("</ol>\n");
        return (html, next_index_after_continuation);
    }
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
                if list.indent == current_indent
                    && (list.start == expected_start
                        || list.start == 1) // Handle repeated 1. syntax
                    =>
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

fn open_ordered_list_tag(start: usize) -> String {
    if start == 1 {
        String::from("<ol>\n")
    } else {
        format!(r#"<ol start="{start}">"#) + "\n"
    }
}

fn render_heading_html(heading: &HeadingBlock, state: &mut ParserState) -> String {
    let id_attr = heading
        .id
        .as_deref()
        .map(|value| format!(r#" id="{}""#, escape_html(value)))
        .unwrap_or_default();
    format!(
        "<h{level}{id_attr}>{}</h{level}>\n",
        render_inline_nodes(&heading.content.nodes, state),
        level = heading.level
    )
}

fn render_code_block_html(code: &CodeBlock) -> String {
    let lang_attr = code
        .language
        .as_deref()
        .filter(|language| !language.is_empty())
        .map(|language| format!(r#" class="language-{}""#, escape_html(language)))
        .unwrap_or_default();
    format!(
        "<pre><code{lang_attr}>{}</code></pre>\n",
        escape_html(&code.code)
    )
}

fn render_table_html(table: &TableBlock, state: &mut ParserState) -> String {
    let mut html = String::from("<table>\n<thead>\n<tr>\n");

    for (header, alignment) in table.headers.iter().zip(table.alignments.iter()) {
        html.push_str(&format!(
            "<th{}>{}</th>\n",
            table_alignment_attr(*alignment),
            render_inline_nodes(&header.nodes, state)
        ));
    }

    html.push_str("</tr>\n</thead>\n<tbody>\n");
    for row in &table.rows {
        html.push_str("<tr>\n");
        for (cell, alignment) in row.iter().zip(table.alignments.iter()) {
            html.push_str(&format!(
                "<td{}>{}</td>\n",
                table_alignment_attr(*alignment),
                render_inline_nodes(&cell.nodes, state)
            ));
        }
        html.push_str("</tr>\n");
    }
    html.push_str("</tbody>\n</table>\n");
    html
}

fn render_definition_list_html(items: &[DefinitionItem], state: &mut ParserState) -> String {
    let mut html = String::from("<dl>\n");
    for item in items {
        html.push_str(&format!(
            "<dt>{}</dt>\n",
            render_inline_nodes(&item.term.nodes, state)
        ));
        for definition in &item.definitions {
            html.push_str(&format!(
                "<dd>{}</dd>\n",
                render_inline_nodes(&definition.nodes, state)
            ));
        }
    }
    html.push_str("</dl>\n");
    html
}

fn render_task_list_html(items: &[TaskItem], state: &mut ParserState) -> String {
    let mut html = String::from(r#"<ul class="task-list">"#);
    html.push('\n');
    for item in items {
        let checked_attr = if item.checked { " checked" } else { "" };
        html.push_str(&format!(
            r#"<li class="task-list-item"><label><input type="checkbox" disabled{checked_attr}> <span>{}</span></label></li>"#,
            render_inline_nodes(&item.content.nodes, state)
        ));
        html.push('\n');
    }
    html.push_str("</ul>\n");
    html
}

fn render_unordered_list_html(list: &UnorderedListBlock, state: &mut ParserState) -> String {
    let mut html = String::from("<ul>\n");
    for item in &list.items {
        html.push_str(&format!(
            "<li>{}</li>\n",
            render_inline_nodes(&item.content.nodes, state)
        ));
    }
    html.push_str("</ul>\n");
    html
}

fn render_ordered_list_html(list: &OrderedListBlock, state: &mut ParserState) -> String {
    let mut html = open_ordered_list_tag(list.start);
    for item in &list.items {
        html.push_str(&format!(
            "<li>{}</li>\n",
            render_inline_nodes(&item.content.nodes, state)
        ));
    }
    html.push_str("</ol>\n");
    html
}

fn render_paragraph_html(paragraph: &ParagraphBlock, state: &mut ParserState) -> String {
    let mut html = String::new();

    for (index, line) in paragraph.lines.iter().enumerate() {
        if index > 0 {
            if paragraph.lines[index - 1].hard_break {
                html.push_str("<br>\n");
            } else {
                html.push(' ');
            }
        }
        html.push_str(&render_inline_nodes(&line.content.nodes, state));
    }

    format!("<p>{}</p>\n", html)
}

fn render_footnotes(state: &ParserState) -> String {
    if state.footnote_order.is_empty() {
        return String::new();
    }

    let mut html = String::from(r#"<section class="footnotes">"#);
    html.push_str("\n<hr>\n<ol>\n");

    for id in &state.footnote_order {
        let content = state
            .footnote_defs
            .get(id)
            .map(String::as_str)
            .unwrap_or("Missing footnote definition.");
        let rendered = append_footnote_backref(
            parse_markdown_fragment_with_options(content, state.options),
            id,
        );
        html.push_str(&format!(
            r#"<li id="fn:{id}">{rendered}</li>"#,
            id = escape_html(id)
        ));
        html.push('\n');
    }

    html.push_str("</ol>\n</section>\n");
    html
}

fn append_footnote_backref(mut rendered: String, id: &str) -> String {
    let backref = format!(
        r##" <a href="#fnref:{id}" class="footnote-backref" aria-label="Back to reference">↩</a>"##,
        id = escape_html(id)
    );

    if rendered.trim().is_empty() {
        return format!("<p>{}</p>\n", backref.trim());
    }

    if let Some(index) = rendered.rfind("</p>") {
        rendered.insert_str(index, &backref);
        return rendered;
    }

    rendered.push_str(&format!("<p>{}</p>\n", backref.trim()));
    rendered
}

fn table_alignment_attr(alignment: Option<Alignment>) -> &'static str {
    match alignment {
        Some(Alignment::Left) => r#" style="text-align: left;""#,
        Some(Alignment::Center) => r#" style="text-align: center;""#,
        Some(Alignment::Right) => r#" style="text-align: right;""#,
        None => "",
    }
}
