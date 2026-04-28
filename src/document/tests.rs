use super::parse_support::is_continuation_line;
use super::{parse_document_fragment, Block};
use crate::inline::InlineNode;

#[test]
fn paragraph_content_preserves_soft_and_hard_breaks() {
    let document = parse_document_fragment("alpha\nbeta  \ngamma");
    let paragraph = match &document.blocks[0] {
        Block::Paragraph(paragraph) => paragraph,
        block => panic!("expected paragraph, got {block:?}"),
    };

    assert_eq!(paragraph.content.source, "alpha\nbeta  \ngamma");
}

#[test]
fn paragraph_content_resolves_reference_links_once_for_whole_paragraph() {
    let document =
        parse_document_fragment("[first line]\n[docs][guide]\n\n[guide]: https://example.com");
    let paragraph = match &document.blocks[0] {
        Block::Paragraph(paragraph) => paragraph,
        block => panic!("expected paragraph, got {block:?}"),
    };

    assert_eq!(paragraph.content.source, "[first line]\n[docs][guide]");
    assert!(matches!(
        paragraph.content.nodes.as_slice(),
        [
            InlineNode::Text(_),
            InlineNode::Link { url, .. }
        ] if url == "https://example.com"
    ));
}

#[test]
fn continuation_line_indented_text() {
    assert!(is_continuation_line("   some continuation text"));
    assert!(is_continuation_line("  more text"));
    assert!(is_continuation_line("\tindented with tab"));
}

#[test]
fn continuation_line_rejects_unindented() {
    assert!(!is_continuation_line("no indent"));
    assert!(!is_continuation_line(""));
}

#[test]
fn continuation_line_rejects_blank() {
    assert!(!is_continuation_line("   "));
    assert!(!is_continuation_line("  \t  "));
}

#[test]
fn continuation_line_rejects_list_items() {
    assert!(!is_continuation_line("  - list item"));
    assert!(!is_continuation_line("  * list item"));
    assert!(!is_continuation_line("  1. ordered item"));
}

#[test]
fn continuation_line_rejects_unindented_headings() {
    assert!(!is_continuation_line("# heading"));
}

#[test]
fn continuation_line_accepts_indented_hash() {
    assert!(is_continuation_line("  # not a heading"));
}

#[test]
fn ordered_list_simple_items() {
    let document = parse_document_fragment("1. first\n2. second\n3. third");
    assert_eq!(document.blocks.len(), 1);
    let list = match &document.blocks[0] {
        Block::OrderedList(list) => list,
        block => panic!("expected ordered list, got {block:?}"),
    };
    let items = &list.items;
    assert_eq!(list.start, 1);
    assert_eq!(list.indent, 0);
    assert_eq!(items.len(), 3);
    assert_eq!(items[0].content.source, "first");
    assert_eq!(items[1].content.source, "second");
    assert_eq!(items[2].content.source, "third");
}

#[test]
fn ordered_list_continuation_line_merges_into_item() {
    let document = parse_document_fragment("1. first item with\n   continuation text\n2. second");
    assert_eq!(
        document.blocks.len(),
        1,
        "should be a single OrderedList, got: {:#?}",
        document.blocks
    );
    let list = match &document.blocks[0] {
        Block::OrderedList(list) => list,
        block => panic!("expected ordered list, got {block:?}"),
    };
    let items = &list.items;
    assert_eq!(items.len(), 2, "items: {items:#?}");
    assert!(
        items[0].content.source.contains("continuation text"),
        "first item should contain continuation: {:?}",
        items[0].content.source
    );
    assert_eq!(items[1].content.source, "second");
}

#[test]
fn ordered_list_lazy_continuation_line_merges_into_item() {
    let document = parse_document_fragment(
        "1. Fix correctness first.\nOnly emit file results.\n2. Restore tree metadata.\nCompute percentages recursively.",
    );
    assert_eq!(
        document.blocks.len(),
        1,
        "should be a single OrderedList, got: {:#?}",
        document.blocks
    );
    let list = match &document.blocks[0] {
        Block::OrderedList(list) => list,
        block => panic!("expected ordered list, got {block:?}"),
    };
    let items = &list.items;
    assert_eq!(items.len(), 2, "items: {items:#?}");
    assert_eq!(
        items[0].content.source,
        "Fix correctness first. Only emit file results."
    );
    assert_eq!(
        items[1].content.source,
        "Restore tree metadata. Compute percentages recursively."
    );
}

#[test]
fn ordered_list_all_ones_syntax_still_produces_single_block() {
    let document = parse_document_fragment("1. alpha\n1. beta\n1. gamma");
    assert_eq!(document.blocks.len(), 1);
    let list = match &document.blocks[0] {
        Block::OrderedList(list) => list,
        block => panic!("expected ordered list, got {block:?}"),
    };
    let items = &list.items;
    assert_eq!(items.len(), 3);
}

#[test]
fn ordered_list_blank_line_between_items_keeps_single_block() {
    let document = parse_document_fragment("1. first\n\n2. second\n\n3. third");
    assert_eq!(
        document.blocks.len(),
        1,
        "blank-separated items should be one OrderedList: {:#?}",
        document.blocks
    );
    let list = match &document.blocks[0] {
        Block::OrderedList(list) => list,
        block => panic!("expected ordered list, got {block:?}"),
    };
    let items = &list.items;
    assert_eq!(items.len(), 3);
}

#[test]
fn ordered_list_blank_line_then_paragraph_continuation_stays_in_same_list() {
    let input = "\
1. Fix correctness first.
Only emit file results.
2. Restore tree metadata invariants after scan.
Either:
- compute `percent` recursively in a post-pass alongside `recalculate_sizes`, or
- compute percentages per-directory when combining children.

The post-pass is cleaner since parallel modes already do a post-pass.

3. Reduce parallelism granularity aggressively.
The current design is trying to parallelize every subtree and every file list.";
    let document = parse_document_fragment(input);

    assert_eq!(document.blocks.len(), 4, "blocks: {:#?}", document.blocks);
    let first_list = match &document.blocks[0] {
        Block::OrderedList(list) => list,
        block => panic!("expected first ordered list, got {block:?}"),
    };
    assert_eq!(first_list.start, 1);
    assert_eq!(first_list.indent, 0);
    assert_eq!(first_list.items.len(), 2, "list: {first_list:#?}");
    let second_list = match &document.blocks[3] {
        Block::OrderedList(list) => list,
        block => panic!("expected second ordered list, got {block:?}"),
    };
    assert_eq!(second_list.start, 3);
    assert_eq!(second_list.indent, 0);
    assert_eq!(second_list.items.len(), 1, "list: {second_list:#?}");
}

#[test]
fn unordered_list_simple_items() {
    let document = parse_document_fragment("- alpha\n- beta\n- gamma");
    assert_eq!(document.blocks.len(), 1);
    let list = match &document.blocks[0] {
        Block::UnorderedList(list) => list,
        block => panic!("expected unordered list, got {block:?}"),
    };
    assert_eq!(list.indent, 0);
    assert_eq!(list.items.len(), 3);
}

#[test]
fn unordered_list_continuation_line_merges_into_item() {
    let document = parse_document_fragment("- first item with\n  continuation text\n- second");
    assert_eq!(
        document.blocks.len(),
        1,
        "should be a single UnorderedList, got: {:#?}",
        document.blocks
    );
    let list = match &document.blocks[0] {
        Block::UnorderedList(list) => list,
        block => panic!("expected unordered list, got {block:?}"),
    };
    let items = &list.items;
    assert_eq!(items.len(), 2, "items: {items:#?}");
    assert!(
        items[0].content.source.contains("continuation text"),
        "first item should contain continuation: {:?}",
        items[0].content.source
    );
}

#[test]
fn unordered_list_with_blank_lines_between() {
    let document = parse_document_fragment("- alpha\n\n- beta\n\n- gamma");
    assert_eq!(document.blocks.len(), 1);
    let list = match &document.blocks[0] {
        Block::UnorderedList(list) => list,
        block => panic!("expected unordered list, got {block:?}"),
    };
    assert_eq!(list.items.len(), 3);
}

#[test]
fn ordered_list_with_sub_bullets_stays_single_list() {
    let input = "\
1. **First item**
   - sub-bullet one
   - sub-bullet two
2. **Second item**
   - another sub-bullet
3. **Third item**";
    let document = parse_document_fragment(input);
    let list = match &document.blocks[0] {
        Block::OrderedList(list) => list,
        block => panic!("expected OrderedList, got {block:?}"),
    };
    assert_eq!(list.items.len(), 1, "list: {list:#?}");
    let nested_first = match &document.blocks[1] {
        Block::UnorderedList(list) => list,
        block => panic!("expected nested unordered list, got {block:?}"),
    };
    assert_eq!(nested_first.indent, 3);
    assert_eq!(nested_first.items.len(), 2);
    let second = match &document.blocks[2] {
        Block::OrderedList(list) => list,
        block => panic!("expected second ordered list, got {block:?}"),
    };
    assert_eq!(second.start, 2);
    assert_eq!(second.items.len(), 1);
    let nested_second = match &document.blocks[3] {
        Block::UnorderedList(list) => list,
        block => panic!("expected second nested unordered list, got {block:?}"),
    };
    assert_eq!(nested_second.indent, 3);
    assert_eq!(nested_second.items.len(), 1);
    let third = match &document.blocks[4] {
        Block::OrderedList(list) => list,
        block => panic!("expected third ordered list, got {block:?}"),
    };
    assert_eq!(third.start, 3);
    assert_eq!(third.items.len(), 1);
}

#[test]
fn ordered_list_sub_bullets_absorbed_into_item_content() {
    let input = "\
1. Main item
   - detail A
   - detail B
2. Second item";
    let document = parse_document_fragment(input);
    let list = match &document.blocks[0] {
        Block::OrderedList(list) => list,
        block => panic!("expected OrderedList, got {block:?}"),
    };
    assert_eq!(list.items.len(), 1, "list: {list:#?}");
    let nested = match &document.blocks[1] {
        Block::UnorderedList(list) => list,
        block => panic!("expected nested unordered list, got {block:?}"),
    };
    assert_eq!(nested.indent, 3);
    assert_eq!(nested.items.len(), 2, "nested: {nested:#?}");
    let trailing = match &document.blocks[2] {
        Block::OrderedList(list) => list,
        block => panic!("expected trailing ordered list, got {block:?}"),
    };
    assert_eq!(trailing.start, 2);
    assert_eq!(trailing.items.len(), 1);
}

#[test]
fn unordered_list_with_indented_sub_bullets_stays_single_list() {
    let input = "\
- First
  - nested under first
- Second
  - nested under second
- Third";
    let document = parse_document_fragment(input);
    let first = match &document.blocks[0] {
        Block::UnorderedList(list) => list,
        block => panic!("expected first unordered list, got {block:?}"),
    };
    assert_eq!(first.items.len(), 1, "first: {first:#?}");
    let nested_first = match &document.blocks[1] {
        Block::UnorderedList(list) => list,
        block => panic!("expected first nested unordered list, got {block:?}"),
    };
    assert_eq!(nested_first.indent, 2);
    assert_eq!(nested_first.items.len(), 1);
    let second = match &document.blocks[2] {
        Block::UnorderedList(list) => list,
        block => panic!("expected second unordered list, got {block:?}"),
    };
    assert_eq!(second.items.len(), 1);
    let nested_second = match &document.blocks[3] {
        Block::UnorderedList(list) => list,
        block => panic!("expected second nested unordered list, got {block:?}"),
    };
    assert_eq!(nested_second.indent, 2);
    assert_eq!(nested_second.items.len(), 1);
    let third = match &document.blocks[4] {
        Block::UnorderedList(list) => list,
        block => panic!("expected third unordered list, got {block:?}"),
    };
    assert_eq!(third.items.len(), 1);
}

#[test]
fn ordered_list_with_double_digit_items_preserves_all_entries() {
    let input = (1..=15)
        .map(|i| format!("{i}. Item {i}"))
        .collect::<Vec<_>>()
        .join("\n");
    let document = parse_document_fragment(&input);
    let list = match &document.blocks[0] {
        Block::OrderedList(list) => list,
        block => panic!("expected ordered list, got {block:?}"),
    };
    assert_eq!(list.start, 1);
    assert_eq!(list.items.len(), 15, "list: {list:#?}");
    for (i, item) in list.items.iter().enumerate() {
        let text = item.content.nodes.iter().find_map(|n| match n {
            crate::document::InlineNode::Text(t) => Some(t.as_str()),
            _ => None,
        });
        assert_eq!(
            text,
            Some(format!("Item {}", i + 1).as_str()),
            "item {i} content mismatch"
        );
    }
}
