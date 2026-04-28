use super::{
    Block, CodeBlock, DefinitionItem, FootnoteDefinition, HeadingBlock, InlineContent,
    LinkReferenceDefinition, ListItem, MarkdownDocument, ParagraphBlock, TableBlock, TaskItem,
};
use crate::inline::InlineNode;
use std::collections::{HashMap, VecDeque};

const DOCUMENT_CACHE_MAX_ENTRIES: usize = 128;
const DOCUMENT_CACHE_MAX_BYTES: usize = 8 * 1024 * 1024;

pub(super) fn insert_cached_document(
    documents: &mut HashMap<String, MarkdownDocument>,
    order: &mut VecDeque<String>,
    used_bytes: &mut usize,
    input: &str,
    document: &MarkdownDocument,
) {
    if let Some(previous) = documents.insert(input.to_string(), document.clone()) {
        *used_bytes = used_bytes.saturating_sub(cached_document_bytes(input, &previous));
    }
    touch_cache_order(order, input);
    *used_bytes = used_bytes.saturating_add(cached_document_bytes(input, document));
    evict_cached_documents(documents, order, used_bytes);
}

pub(super) fn touch_cache_order(order: &mut VecDeque<String>, input: &str) {
    order.retain(|candidate| candidate != input);
    order.push_back(input.to_string());
}

fn evict_cached_documents(
    documents: &mut HashMap<String, MarkdownDocument>,
    order: &mut VecDeque<String>,
    used_bytes: &mut usize,
) {
    while documents.len() > DOCUMENT_CACHE_MAX_ENTRIES || *used_bytes > DOCUMENT_CACHE_MAX_BYTES {
        let Some(evicted_key) = order.pop_front() else {
            break;
        };
        if let Some(evicted) = documents.remove(&evicted_key) {
            *used_bytes = used_bytes.saturating_sub(cached_document_bytes(&evicted_key, &evicted));
        }
    }
}

fn cached_document_bytes(input: &str, document: &MarkdownDocument) -> usize {
    input
        .len()
        .saturating_add(markdown_document_bytes(document))
        .saturating_add(64)
}

fn markdown_document_bytes(document: &MarkdownDocument) -> usize {
    let mut total = 128usize;
    for block in &document.blocks {
        total = total.saturating_add(block_bytes(block));
    }
    for FootnoteDefinition { id, content } in &document.footnotes {
        total = total
            .saturating_add(id.len())
            .saturating_add(content.len())
            .saturating_add(32);
    }
    for LinkReferenceDefinition { label, url, title } in &document.references {
        total = total
            .saturating_add(label.len())
            .saturating_add(url.len())
            .saturating_add(title.as_deref().map(str::len).unwrap_or(0))
            .saturating_add(32);
    }
    total
}

fn block_bytes(block: &Block) -> usize {
    match block {
        Block::Paragraph(paragraph) => paragraph_block_bytes(paragraph),
        Block::Heading(HeadingBlock { content, id, .. }) => id
            .as_deref()
            .map(str::len)
            .unwrap_or(0)
            .saturating_add(inline_content_bytes(content))
            .saturating_add(24),
        Block::CodeBlock(CodeBlock { language, code }) => language
            .as_deref()
            .map(str::len)
            .unwrap_or(0)
            .saturating_add(code.len())
            .saturating_add(24),
        Block::Table(table) => table_block_bytes(table),
        Block::BlockQuote(blocks) => {
            let mut total = 24usize;
            for block in blocks {
                total = total.saturating_add(block_bytes(block));
            }
            total
        }
        Block::DefinitionList(items) => {
            let mut total = 24usize;
            for DefinitionItem { term, definitions } in items {
                total = total.saturating_add(inline_content_bytes(term));
                for definition in definitions {
                    total = total.saturating_add(inline_content_bytes(definition));
                }
            }
            total
        }
        Block::TaskList(items) => {
            let mut total = 24usize;
            for TaskItem { content, .. } in items {
                total = total.saturating_add(inline_content_bytes(content));
            }
            total
        }
        Block::UnorderedList(list) => list_block_bytes(&list.items),
        Block::OrderedList(list) => list_block_bytes(&list.items).saturating_add(16),
        Block::HorizontalRule => 8,
    }
}

fn paragraph_block_bytes(paragraph: &ParagraphBlock) -> usize {
    let mut total = inline_content_bytes(&paragraph.content).saturating_add(24);
    for line in &paragraph.lines {
        total = total
            .saturating_add(inline_content_bytes(&line.content))
            .saturating_add(8);
    }
    total
}

fn table_block_bytes(table: &TableBlock) -> usize {
    let mut total = 24usize;
    for header in &table.headers {
        total = total.saturating_add(inline_content_bytes(header));
    }
    for row in &table.rows {
        for cell in row {
            total = total.saturating_add(inline_content_bytes(cell));
        }
    }
    total.saturating_add(table.alignments.len() * 8)
}

fn list_block_bytes(items: &[ListItem]) -> usize {
    let mut total = 24usize;
    for item in items {
        total = total.saturating_add(inline_content_bytes(&item.content));
    }
    total
}

fn inline_content_bytes(content: &InlineContent) -> usize {
    content
        .source
        .len()
        .saturating_add(inline_nodes_bytes(&content.nodes))
        .saturating_add(16)
}

fn inline_nodes_bytes(nodes: &[InlineNode]) -> usize {
    let mut total = 0usize;
    for node in nodes {
        total = total.saturating_add(inline_node_bytes(node));
    }
    total
}

fn inline_node_bytes(node: &InlineNode) -> usize {
    match node {
        InlineNode::Text(text)
        | InlineNode::Entity(text)
        | InlineNode::Code(text)
        | InlineNode::FootnoteReference(text)
        | InlineNode::RawHtml(text) => text.len().saturating_add(16),
        InlineNode::Emphasis(children)
        | InlineNode::Strong(children)
        | InlineNode::Strikethrough(children)
        | InlineNode::Mark(children)
        | InlineNode::Subscript(children)
        | InlineNode::Superscript(children) => inline_nodes_bytes(children).saturating_add(16),
        InlineNode::Link {
            children,
            url,
            title,
        } => inline_nodes_bytes(children)
            .saturating_add(url.len())
            .saturating_add(title.as_deref().map(str::len).unwrap_or(0))
            .saturating_add(24),
        InlineNode::Image { alt, url, title } => alt
            .len()
            .saturating_add(url.len())
            .saturating_add(title.as_deref().map(str::len).unwrap_or(0))
            .saturating_add(24),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::document::DocumentCache;

    #[test]
    fn document_cache_evicts_fragment_entries_past_entry_cap() {
        let mut cache = DocumentCache::default();
        for idx in 0..=DOCUMENT_CACHE_MAX_ENTRIES {
            let input = format!("entry-{idx}");
            let _ = cache.parse_document_fragment(&input);
        }

        assert_eq!(cache.fragment_documents.len(), DOCUMENT_CACHE_MAX_ENTRIES);
        assert!(!cache.fragment_documents.contains_key("entry-0"));
        assert!(cache
            .fragment_documents
            .contains_key(&format!("entry-{DOCUMENT_CACHE_MAX_ENTRIES}")));
    }

    #[test]
    fn document_cache_evicts_large_fragments_when_over_byte_cap() {
        let mut cache = DocumentCache::default();
        let large = "x".repeat((DOCUMENT_CACHE_MAX_BYTES / 6) + 1024);
        let first = format!("first\n\n{large}");
        let second = format!("second\n\n{large}");
        let third = format!("third\n\n{large}");

        let _ = cache.parse_document_fragment(&first);
        let _ = cache.parse_document_fragment(&second);
        let _ = cache.parse_document_fragment(&third);

        assert!(cache.fragment_document_bytes <= DOCUMENT_CACHE_MAX_BYTES);
        assert!(!cache.fragment_documents.contains_key(&first));
        assert!(cache.fragment_documents.contains_key(&third));
    }
}
