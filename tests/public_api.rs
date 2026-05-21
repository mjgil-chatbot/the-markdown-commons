use tmc::{
    parse_document, parse_document_cached, parse_inline_nodes, parse_markdown,
    parse_markdown_cached, parse_markdown_fragment, parse_markdown_fragment_cached,
    parse_markdown_fragment_with_options, parse_markdown_fragment_with_options_cached,
    parse_markdown_with_options, parse_markdown_with_options_cached, Block, DocumentCache,
    HtmlOptions, InlineNode,
};

#[test]
fn root_html_api_parses_markdown() {
    let html = parse_markdown("# Title\n\nThis is **bold**.");

    assert!(html.contains("<h1>Title</h1>"));
    assert!(html.contains("<strong>bold</strong>"));
}

#[test]
fn root_fragment_api_skips_footnote_section() {
    let html = parse_markdown_fragment("Note[^1].\n\n[^1]: Footnote.");

    assert!(
        html.contains(r##"<sup id="fnref:1"><a href="#fn:1" class="footnote-ref">1</a></sup>"##)
    );
    assert!(!html.contains(r#"<section class="footnotes">"#));
}

#[test]
fn module_and_root_html_api_match() {
    let markdown = "| A | B |\n| --- | --- |\n| 1 | 2 |";

    assert_eq!(
        parse_markdown(markdown),
        tmc::html::parse_markdown(markdown)
    );
}

#[test]
fn module_and_root_fragment_api_match() {
    let markdown = "Inline [link](https://example.com) and `code`.";

    assert_eq!(
        parse_markdown_fragment(markdown),
        tmc::html::parse_markdown_fragment(markdown)
    );
}

#[test]
fn root_and_module_option_apis_match() {
    let markdown = "<em>raw</em> [Guide](https://example.com)";
    let options = HtmlOptions {
        link_target_blank: false,
        allow_raw_html: false,
    };

    assert_eq!(
        parse_markdown_with_options(markdown, options),
        tmc::html::parse_markdown_with_options(markdown, options)
    );
    assert_eq!(
        parse_markdown_fragment_with_options(markdown, options),
        tmc::html::parse_markdown_fragment_with_options(markdown, options)
    );
}

#[test]
fn typed_document_api_exposes_shared_blocks() {
    let document = parse_document("# Title\n\n| A | B |\n| --- | --- |\n| 1 | 2 |");

    assert!(
        matches!(document.blocks.first(), Some(Block::Heading(heading)) if !heading.content.nodes.is_empty())
    );
    assert!(matches!(document.blocks.get(1), Some(Block::Table(_))));
}

#[test]
fn rich_document_model_stores_inline_content_in_lists_and_tables() {
    let document = parse_document(
        "- [Guide](https://example.com)\n\n| Name | Value |\n| --- | --- |\n| **speed** | `fast` |",
    );

    assert!(matches!(
        document.blocks.first(),
        Some(Block::UnorderedList(list))
            if matches!(list.items[0].content.nodes.first(), Some(InlineNode::Link { .. }))
    ));
    assert!(matches!(
        document.blocks.get(1),
        Some(Block::Table(table))
            if matches!(table.rows[0][0].nodes.first(), Some(InlineNode::Strong(_)))
                && matches!(table.rows[0][1].nodes.first(), Some(InlineNode::Code(_)))
    ));
}

#[test]
fn document_cache_matches_direct_parse() {
    let markdown = "# Title\n\nParagraph with **bold**.";
    let mut cache = DocumentCache::default();

    let direct = parse_document(markdown);
    let cached_once = parse_document_cached(&mut cache, markdown);
    let cached_twice = parse_document_cached(&mut cache, markdown);

    assert_eq!(direct, cached_once);
    assert_eq!(cached_once, cached_twice);
}

#[test]
fn html_cache_wrappers_match_direct_html_helpers() {
    let markdown = "# Title\n\nParagraph with **bold**.";
    let mut cache = DocumentCache::default();
    let options = HtmlOptions::default();

    assert_eq!(
        parse_markdown(markdown),
        parse_markdown_cached(&mut cache, markdown)
    );
    assert_eq!(
        parse_markdown_with_options(markdown, options),
        parse_markdown_with_options_cached(&mut cache, markdown, options)
    );
    assert_eq!(
        parse_markdown_fragment(markdown),
        parse_markdown_fragment_cached(&mut cache, markdown)
    );
    assert_eq!(
        parse_markdown_fragment_with_options(markdown, options),
        parse_markdown_fragment_with_options_cached(&mut cache, markdown, options)
    );
}

#[test]
fn shared_inline_api_parses_links_and_formatting() {
    let nodes = parse_inline_nodes(
        "A [link](https://example.com) with **bold** text.",
        &Default::default(),
    );

    assert!(nodes
        .iter()
        .any(|node| matches!(node, InlineNode::Link { .. })));
    assert!(nodes
        .iter()
        .any(|node| matches!(node, InlineNode::Strong(_))));
}
