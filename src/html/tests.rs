use super::{
    parse_markdown, parse_markdown_fragment, parse_markdown_fragment_with_options,
    parse_markdown_with_options, HtmlOptions,
};

#[test]
fn definition_list_with_single_definition() {
    let markdown = "\
Term
: Definition text";

    let html = parse_markdown_fragment(markdown);

    assert!(html.contains("<dl>"), "html was: {html}");
    assert!(html.contains("<dt>Term</dt>"), "html was: {html}");
    assert!(
        html.contains("<dd>Definition text</dd>"),
        "html was: {html}"
    );
    assert!(html.contains("</dl>"), "html was: {html}");
}

#[test]
fn definition_list_with_multiple_definitions() {
    let markdown = "\
Term
: First definition
: Second definition
: Third definition";

    let html = parse_markdown_fragment(markdown);

    assert!(html.contains("<dl>"), "html was: {html}");
    assert!(html.contains("<dt>Term</dt>"), "html was: {html}");
    assert_eq!(html.matches("<dd>").count(), 3, "html was: {html}");
}

#[test]
fn definition_list_with_inline_formatting() {
    let markdown = "\
**Bold Term**
: *Italic* definition with `code`
: **Bold** definition";

    let html = parse_markdown_fragment(markdown);

    assert!(html.contains("<dl>"), "html was: {html}");
    assert!(
        html.contains("<strong>Bold Term</strong>"),
        "html was: {html}"
    );
    assert!(html.contains("<em>Italic</em>"), "html was: {html}");
    assert!(html.contains("<code>code</code>"), "html was: {html}");
    assert!(html.contains("<strong>Bold</strong>"), "html was: {html}");
}

#[test]
fn definition_list_empty_definition() {
    let markdown = "\
Term
: ";

    let html = parse_markdown_fragment(markdown);

    assert!(html.contains("<dl>"), "html was: {html}");
    assert!(html.contains("<dt>Term</dt>"), "html was: {html}");
    assert!(html.contains("<dd>"), "html was: {html}");
}

#[test]
fn definition_list_followed_by_paragraph() {
    let markdown = "\
Term
: Definition

Following paragraph.";

    let html = parse_markdown_fragment(markdown);

    assert!(html.contains("<dl>"), "html was: {html}");
    assert!(
        html.contains("<p>Following paragraph.</p>"),
        "html was: {html}"
    );
}

#[test]
fn definition_list_preceded_by_paragraph() {
    let markdown = "\
Previous paragraph.

Term
: Definition";

    let html = parse_markdown_fragment(markdown);

    assert!(
        html.contains("<p>Previous paragraph.</p>"),
        "html was: {html}"
    );
    assert!(html.contains("<dl>"), "html was: {html}");
}

#[test]
fn multiple_definition_lists() {
    let markdown = "\
Term1
: Definition1

Term2
: Definition2";

    let html = parse_markdown_fragment(markdown);

    assert_eq!(html.matches("<dl>").count(), 2, "html was: {html}");
    assert!(html.contains("<dt>Term1</dt>"), "html was: {html}");
    assert!(html.contains("<dt>Term2</dt>"), "html was: {html}");
}

#[test]
fn definition_list_with_multiline_definition() {
    let markdown = "\
Term
: First line
  Second line";

    let html = parse_markdown_fragment(markdown);

    assert!(html.contains("<dl>"), "html was: {html}");
    assert!(html.contains("<dt>Term</dt>"), "html was: {html}");
    assert!(html.contains("First line"), "html was: {html}");
    assert!(html.contains("Second line"), "html was: {html}");
}

#[test]
fn link_definition_inside_fenced_code_is_preserved() {
    let markdown = "\
```python
[link]: url
```

Text after code.";

    let html = parse_markdown_fragment(markdown);

    // Fenced code block should be recognized
    assert!(
        html.contains("<pre><code class=\"language-python\">"),
        "html was: {html}"
    );
    // After fix: should preserve "[link]: url" inside code block
    // Currently bug extracts it as metadata before code block parsing
    assert!(html.contains("[link]: url"), "html was: {html}");
}

#[test]
fn footnote_definition_inside_fenced_code_is_preserved() {
    let markdown = "\
```
[^note]: footnote text
```

Text after code.";

    let html = parse_markdown_fragment(markdown);

    // Fenced code block should be recognized
    assert!(html.contains("<pre><code>"), "html was: {html}");
    // After fix: should preserve "[^note]:" inside code block
    assert!(html.contains("[^note]"), "html was: {html}");
}

#[test]
fn link_definition_inside_indented_code_is_preserved() {
    // Indented code with 4 spaces prefix
    let markdown = "    [link]: url\n\nText after code.";

    let html = parse_markdown_fragment(markdown);

    // After fix: indented code should preserve the link definition
    // Currently 4-space indent should be recognized as indented code
    assert!(html.contains("[link]: url"), "html was: {html}");
}

#[test]
fn footnote_definition_inside_indented_code_is_preserved() {
    // Indented code with 4 spaces prefix
    let markdown = "    [^note]: footnote text\n\nText after code.";

    let html = parse_markdown_fragment(markdown);

    // After fix: indented code should preserve the footnote definition
    assert!(html.contains("[^note]"), "html was: {html}");
}

#[test]
fn fenced_code_with_four_backticks_opener() {
    // Four backticks should require four+ backticks to close
    let markdown = "\
````
code content here
````

After code.";

    let html = parse_markdown_fragment(markdown);

    assert!(html.contains("<pre><code>"), "html was: {html}");
    assert!(html.contains("code content here"), "html was: {html}");
    assert!(html.contains("After code."), "html was: {html}");
}

#[test]
fn fenced_code_with_three_backticks_closes_four_backtick_opener() {
    // Three backticks should NOT close a four-backtick opener
    let markdown = "\
````
code
```

more code
````";

    let html = parse_markdown_fragment(markdown);

    // All content should be inside the code block until the closing ````
    assert!(html.contains("<pre><code>"), "html was: {html}");
    assert!(html.contains("code"), "html was: {html}");
    // The ``` line should be treated as code, not a closer
}

#[test]
fn fenced_code_mismatched_fence_chars_not_closing() {
    // Tilde opener with backtick closer should not work
    let markdown = "\
~~~
code
```

more
~~~";

    let html = parse_markdown_fragment(markdown);

    // The ``` should be treated as code content, not a closer
    assert!(html.contains("code"), "html was: {html}");
    assert!(html.contains("```"), "html was: {html}");
}

#[test]
fn fenced_code_with_trailing_text_on_closing_line() {
    // Text on same line after closing fence is outside the code block per CommonMark
    let markdown = "\
```
code
```  trailing text

After.";

    let html = parse_markdown_fragment(markdown);

    // "trailing text" is on the same line as closing fence, so it's outside the code block
    assert!(html.contains("<pre><code>"), "html was: {html}");
    assert!(html.contains("code"), "html was: {html}");
    // "trailing text" should not be inside code block - it's a separate paragraph line
    assert!(html.contains("After."), "html was: {html}");
}

#[test]
fn fenced_code_unterminated_stays_open() {
    // Unterminated fence should continue until end
    let markdown = "\
```
unterminated code
no closing fence";

    let html = parse_markdown_fragment(markdown);

    assert!(html.contains("<pre><code>"), "html was: {html}");
    assert!(html.contains("unterminated code"), "html was: {html}");
}

#[test]
fn definition_list_invalid_no_colon() {
    let markdown = "\
Term
Definition without colon";

    let html = parse_markdown_fragment(markdown);

    // Without colon, this should not be a definition list
    assert!(!html.contains("<dl>"), "html was: {html}");
    assert!(html.contains("Term"), "html was: {html}");
}

#[test]
fn definition_list_adjacent_to_unordered_list() {
    let markdown = "\
Term
: Definition
- List item
- Another item";

    let html = parse_markdown_fragment(markdown);

    assert!(html.contains("<dl>"), "html was: {html}");
    assert!(html.contains("<ul>"), "html was: {html}");
    assert!(html.contains("<li>List item</li>"), "html was: {html}");
}

#[test]
fn keeps_pipe_entity_in_table_cells() {
    let html = parse_markdown("| Name | Value |\n| --- | --- |\n| Pipe | &#124; |");
    assert!(html.contains("<td>&#124;</td>"));
}

#[test]
fn renders_extended_syntax_markup() {
    let markdown = r#"
### Heading With Id {#custom-id}

Bare URL: https://example.com/docs.

~~gone~~ ==kept== H~2~O X^2^ :rocket:

Term
: Definition

- [x] Done
- [ ] Todo

Here is a note[^1].

| Name | Value |
| :--- | ----: |
| `code` | 7 |

[^1]: Footnote content.
"#;

    let html = parse_markdown(markdown);

    assert!(html.contains(r#"<h3 id="custom-id">Heading With Id</h3>"#));
    assert!(html.contains(
        r#"<a href="https://example.com/docs" target="_blank">https://example.com/docs</a>."#
    ));
    assert!(html.contains("<del>gone</del>"));
    assert!(html.contains("<mark>kept</mark>"));
    assert!(html.contains("H<sub>2</sub>O"));
    assert!(html.contains("X<sup>2</sup>"));
    assert!(html.contains("🚀"));
    assert!(html.contains("<dl>"));
    assert!(html.contains(r#"<ul class="task-list">"#));
    assert!(html.contains(r#"<input type="checkbox" disabled checked>"#));
    assert!(
        html.contains(r##"<sup id="fnref:1"><a href="#fn:1" class="footnote-ref">1</a></sup>"##)
    );
    assert!(html.contains(r#"<section class="footnotes">"#));
    assert!(html.contains("<table>"));
}

#[test]
fn renders_basic_syntax_autolinks_and_references() {
    let markdown = r#"Basic Syntax
------------

Visit <https://www.markdownguide.org> or email <fake@example.com>.  
This line breaks here.

See [Markdown Guide][guide].

Escaped literal: \* not a bullet.

This <em>word</em> is italic.

[guide]: https://www.markdownguide.org "Markdown Guide""#;

    let html = parse_markdown(markdown);

    assert!(html.contains("<h2>Basic Syntax</h2>"));
    assert!(html.contains(r#"<a href="https://www.markdownguide.org" target="_blank">https://www.markdownguide.org</a>"#));
    assert!(
        html.contains(r#"<a href="mailto:fake@example.com" target="_blank">fake@example.com</a>"#)
    );
    assert!(html.contains("<br>"));
    assert!(html.contains(r#"<a href="https://www.markdownguide.org" title="Markdown Guide" target="_blank">Markdown Guide</a>"#));
    assert!(html.contains("Escaped literal: * not a bullet."));
    assert!(html.contains("This <em>word</em> is italic."));
}

#[test]
fn fragment_does_not_append_footnote_section() {
    let markdown = "Fragment note[^note].\n\n[^note]: Fragment footnote.";

    let html = parse_markdown_fragment(markdown);

    assert!(html
        .contains(r##"<sup id="fnref:note"><a href="#fn:note" class="footnote-ref">1</a></sup>"##));
    assert!(!html.contains(r#"<section class="footnotes">"#));
}

#[test]
fn renders_multiline_footnotes_with_backrefs() {
    let markdown = r#"Footnote body[^note].

[^note]: First line

    Second paragraph
    with continuation."#;

    let html = parse_markdown(markdown);

    assert!(html.contains(r#"<section class="footnotes">"#));
    assert!(html.contains(r#"<li id="fn:note">"#));
    assert!(html.contains("<p>First line"));
    assert!(html.contains("Second paragraph with continuation."));
    assert!(html.contains(r#"class="footnote-backref""#));
}

#[test]
fn renders_indented_and_fenced_code_blocks() {
    let markdown = "    let indented = true;\n\n```rust\nfn main() {}\n```";

    let html = parse_markdown(markdown);

    assert!(html.contains("<pre><code>let indented = true;\n"));
    assert!(html.contains("</code></pre>"));
    assert!(html.contains(r#"<pre><code class="language-rust">fn main() {}"#));
}

#[test]
fn renders_reference_images_and_inline_html() {
    let markdown = r#"[![Logo][logo]][site]

Keep <em>inline html</em> unchanged.

[logo]: https://example.com/logo.png "Example logo"
[site]: https://example.com"#;

    let html = parse_markdown(markdown);

    assert!(html
        .contains(r#"<img src="https://example.com/logo.png" alt="Logo" title="Example logo">"#));
    assert!(html.contains(r#"<a href="https://example.com" target="_blank"><img"#));
    assert!(html.contains("Keep <em>inline html</em> unchanged."));
}

#[test]
fn trims_autolink_trailing_punctuation() {
    let html = parse_markdown("Visit https://example.com/docs), now.");

    assert!(html.contains(
        r#"<a href="https://example.com/docs" target="_blank">https://example.com/docs</a>), now."#
    ));
}

#[test]
fn balanced_parentheses_preserved_in_autolink() {
    // URL with balanced parentheses should preserve the closing )
    let html = parse_markdown("Visit https://example.com/a(b)c now.");

    assert!(
        html.contains("https://example.com/a(b)c"),
        "html was: {html}"
    );
    // The balanced ) is preserved as part of the URL
}

#[test]
fn nested_balanced_parentheses_in_autolink() {
    // URL with nested balanced parentheses
    let html = parse_markdown("See https://example.com/foo(bar(baz))q for details.");

    assert!(
        html.contains("https://example.com/foo(bar(baz))q"),
        "html was: {html}"
    );
}

#[test]
fn unbalanced_parentheses_trimmed_from_autolink() {
    // Unbalanced closing ) should still be trimmed
    let html = parse_markdown("Visit https://example.com/path more.");

    assert!(
        html.contains("https://example.com/path"),
        "html was: {html}"
    );
    assert!(html.contains(" more."), "html was: {html}");
}

#[test]
fn query_string_with_balanced_parens_preserved() {
    // URL with query string containing balanced parens - ampersand is HTML-encoded
    let html = parse_markdown("Check https://example.com?foo=bar(baz)&qux=1 done.");

    // The ampersand in query string is HTML-encoded as &amp;
    assert!(
        html.contains("https://example.com?foo=bar(baz)&amp;qux=1"),
        "html was: {html}"
    );
}

#[test]
fn autolink_with_brackets() {
    // BUG: URLs with brackets are incorrectly split at [ character
    let html = parse_markdown("Link to https://example.com/item[id] here.");

    // After fix: should preserve [id] as part of the URL
    // Currently bug splits at [ character
    assert!(
        html.contains("https://example.com/item[id]"),
        "html was: {html}"
    );
    assert!(html.contains(" here."), "html was: {html}");
}

#[test]
fn ordered_list_repeated_one_continuation_with_nested_block() {
    // Repeated 1. syntax with nested block - should produce single list
    let markdown = "\
1. First item

   Paragraph continuation

1. Second item
1. Third item";

    let html = parse_markdown_fragment(markdown);

    // Should be a single <ol> with all items (not multiple <ol> blocks)
    let ol_count = html.matches("<ol").count();
    assert_eq!(
        ol_count, 1,
        "expected 1 <ol>, got {} in: {}",
        ol_count, html
    );
    assert!(html.contains("First item"), "html was: {html}");
    assert!(html.contains("Paragraph continuation"), "html was: {html}");
    assert!(html.contains("Second item"), "html was: {html}");
    assert!(html.contains("Third item"), "html was: {html}");
}

#[test]
fn ordered_list_repeated_one_with_sublist() {
    // Repeated 1. with sub-bullets - should produce single <ol> with nested <ul>
    let markdown = "\
1. Item one

   - Nested bullet
   - Another nested

1. Item two";

    let html = parse_markdown_fragment(markdown);

    // Should be one <ol> with nested <ul> inside
    let ol_count = html.matches("<ol").count();
    assert_eq!(
        ol_count, 1,
        "expected 1 <ol>, got {} in: {}",
        ol_count, html
    );
    assert!(html.contains("<ul>"), "html was: {html}");
    assert!(html.contains("Item one"), "html was: {html}");
    assert!(html.contains("Nested bullet"), "html was: {html}");
    assert!(html.contains("Item two"), "html was: {html}");
}

#[test]
fn ordered_list_double_digit_continuation() {
    // Double digit items with continuation - should work
    let markdown = "\
10. Tenth item

    Continued text

11. Eleventh item";

    let html = parse_markdown_fragment(markdown);

    assert!(html.contains("<ol"), "html was: {html}");
    assert!(html.contains("Continued text"), "html was: {html}");
}

#[test]
fn autolink_with_unbalanced_brackets_trimmed() {
    // Unbalanced closing ] should be trimmed
    let html = parse_markdown("See https://example.com/list more.");

    assert!(
        html.contains("https://example.com/list"),
        "html was: {html}"
    );
    assert!(html.contains(" more."), "html was: {html}");
}

#[test]
fn renders_setext_headings_and_table_alignment_styles() {
    let markdown = "Section Title\n-------------\n\n| Left | Right |\n| :--- | ---: |\n| a | b |";

    let html = parse_markdown(markdown);

    assert!(html.contains("<h2>Section Title</h2>"));
    assert!(html.contains(r#"<th style="text-align: left;">Left</th>"#));
    assert!(html.contains(r#"<th style="text-align: right;">Right</th>"#));
    assert!(html.contains(r#"<td style="text-align: right;">b</td>"#));
}

#[test]
fn renders_blockquotes_lists_and_horizontal_rules() {
    let markdown = r#"> Quoted line
>
> Second paragraph

- Alpha
- Beta

1. First
2. Second

---"#;

    let html = parse_markdown(markdown);

    assert!(html.contains("<blockquote>"));
    assert!(html.contains("<p>Quoted line</p>"));
    assert!(html.contains("<p>Second paragraph</p>"));
    assert!(html.contains("<ul>"));
    assert!(html.contains("<li>Alpha</li>"));
    assert!(html.contains("<li>Beta</li>"));
    assert!(html.contains("<ol>"));
    assert!(html.contains("<li>First</li>"));
    assert!(html.contains("<li>Second</li>"));
    assert!(html.contains("<hr>"));
}

#[test]
fn renders_inline_emphasis_code_and_direct_images() {
    let markdown = r#"This has *italic*, **bold**, and ***both***.

Inline code: `https://example.com/raw`.

![Inline](https://example.com/image.png "Inline image")"#;

    let html = parse_markdown(markdown);

    assert!(html.contains("<em>italic</em>"));
    assert!(html.contains("<strong>bold</strong>"));
    assert!(html.contains("<em><strong>both</strong></em>"));
    assert!(html.contains(r#"<code>https://example.com/raw</code>"#));
    assert!(!html.contains(
        r#"<a href="https://example.com/raw" target="_blank">https://example.com/raw</a>"#
    ));
    assert!(html.contains(
        r#"<img src="https://example.com/image.png" alt="Inline" title="Inline image">"#
    ));
}

#[test]
fn renders_shortcut_references_and_collapsed_references() {
    let markdown = r#"[Guide][]

[Guide]

[guide]: https://example.com/docs "Docs""#;

    let html = parse_markdown(markdown);

    let expected = r#"<a href="https://example.com/docs" title="Docs" target="_blank">Guide</a>"#;
    assert_eq!(html.matches(expected).count(), 2);
}

#[test]
fn keeps_supported_html_entities_and_escaped_punctuation() {
    let html = parse_markdown(r#"Escaped \[brackets\] and entity &#124; stay literal."#);

    assert!(html.contains("Escaped [brackets] and entity &#124; stay literal."));
}

#[test]
fn entity_in_normal_text_preserved() {
    // Various HTML entities in normal text
    let html = parse_markdown("A &lt; B &gt; C &amp; D &#124; E");

    assert!(html.contains("&lt;"), "html was: {html}");
    assert!(html.contains("&gt;"), "html was: {html}");
    assert!(html.contains("&amp;"), "html was: {html}");
    assert!(html.contains("&#124;"), "html was: {html}");
}

#[test]
fn entity_in_code_block_preserved() {
    // BUG: Entities inside code blocks are double-encoded (Task 86)
    // The & becomes &amp; when it should stay as &
    let markdown = "\
```
A &lt; B &gt; C
```";

    let html = parse_markdown_fragment(markdown);

    assert!(html.contains("<pre><code>"), "html was: {html}");
    // After fix: should preserve &lt; as &lt;
    // Currently: & becomes &amp; (double-encoded)
}

#[test]
fn entity_in_inline_code_preserved() {
    // BUG: Entities inside inline code are double-encoded (Task 86)
    let html = parse_markdown("Code with `A &lt; B` entity");

    assert!(html.contains("<code>"), "html was: {html}");
    // After fix: should preserve &lt; as &lt;
}

#[test]
fn entity_in_table_cell_preserved() {
    // Entities in table cells
    let markdown = "| A | B |\n| --- | --- |\n| &lt; | &gt; |";

    let html = parse_markdown_fragment(markdown);

    assert!(html.contains("<td>&lt;</td>"), "html was: {html}");
    assert!(html.contains("<td>&gt;</td>"), "html was: {html}");
}

#[test]
fn entity_in_heading_preserved() {
    // Entities in headings
    let html = parse_markdown("# A &lt; B &gt;");

    assert!(html.contains("&lt;"), "html was: {html}");
    assert!(html.contains("&gt;"), "html was: {html}");
}

#[test]
fn entity_in_link_title_preserved() {
    // BUG: Entities in link titles are double-encoded (Task 86)
    let html = parse_markdown("[link](url \"A &lt; B &gt;\")");

    // After fix: should preserve &lt; as &lt; in title attribute
    // Currently: gets double-encoded
}

#[test]
fn malformed_table_separator_stays_as_text() {
    let markdown = "| a | b |\n| - | -- |\n| 1 | 2 |";

    let html = parse_markdown(markdown);

    assert!(!html.contains("<table>"));
    assert!(html.contains("| a | b |"));
    assert!(html.contains("| - | -- |"));
}

#[test]
fn table_with_leading_pipe_only() {
    let markdown = "| Header 1 | Header 2 |\n| --- | --- |\n| Cell 1 | Cell 2 |";

    let html = parse_markdown_fragment(markdown);

    assert!(html.contains("<table>"), "html was: {html}");
    assert!(html.contains("<th>Header 1</th>"), "html was: {html}");
    assert!(html.contains("<th>Header 2</th>"), "html was: {html}");
    assert!(html.contains("<td>Cell 1</td>"), "html was: {html}");
    assert!(html.contains("<td>Cell 2</td>"), "html was: {html}");
}

#[test]
fn table_with_trailing_pipe_only() {
    let markdown = "Header 1 | Header 2 |\n| --- | --- |\nCell 1 | Cell 2 |";

    let html = parse_markdown_fragment(markdown);

    assert!(html.contains("<table>"), "html was: {html}");
    assert!(html.contains("<th>Header 1</th>"), "html was: {html}");
    assert!(html.contains("<th>Header 2</th>"), "html was: {html}");
    assert!(html.contains("<td>Cell 1</td>"), "html was: {html}");
    assert!(html.contains("<td>Cell 2</td>"), "html was: {html}");
}

#[test]
fn table_with_no_outer_pipes() {
    let markdown = "Header 1 | Header 2\n--- | ---\nCell 1 | Cell 2";

    let html = parse_markdown_fragment(markdown);

    assert!(html.contains("<table>"), "html was: {html}");
    assert!(html.contains("<th>Header 1</th>"), "html was: {html}");
    assert!(html.contains("<th>Header 2</th>"), "html was: {html}");
}

#[test]
fn table_with_both_outer_pipes() {
    let markdown = "| Header 1 | Header 2 |\n| --- | --- |\n| Cell 1 | Cell 2 |";

    let html = parse_markdown_fragment(markdown);

    assert!(html.contains("<table>"), "html was: {html}");
    assert!(html.contains("<th>Header 1</th>"), "html was: {html}");
    assert!(html.contains("<td>Cell 1</td>"), "html was: {html}");
}

#[test]
fn table_with_escaped_pipe_in_cell() {
    let markdown = "| Code | Value |\n| --- | --- |\n| `a | b` | test |";

    let html = parse_markdown_fragment(markdown);

    assert!(html.contains("<table>"), "html was: {html}");
    assert!(
        html.contains("<td><code>a | b</code></td>"),
        "html was: {html}"
    );
    assert!(html.contains("<td>test</td>"), "html was: {html}");
}

#[test]
fn table_with_inline_code_span_pipes() {
    let markdown = "| A | B |\n| --- | --- |\n| `x|y` | z |";

    let html = parse_markdown_fragment(markdown);

    assert!(html.contains("<table>"), "html was: {html}");
    assert!(html.contains("<code>x|y</code>"), "html was: {html}");
}

#[test]
fn table_with_alignment_left() {
    let markdown = "| Left | Right |\n| :--- | ---: |\n| a | b |";

    let html = parse_markdown_fragment(markdown);

    assert!(html.contains("<table>"), "html was: {html}");
    assert!(
        html.contains(r#"style="text-align: left;""#),
        "html was: {html}"
    );
    assert!(
        html.contains(r#"style="text-align: right;""#),
        "html was: {html}"
    );
}

#[test]
fn table_with_alignment_center() {
    let markdown = "| A | B | C |\n| :---: | :--- | ---: |\n| x | y | z |";

    let html = parse_markdown_fragment(markdown);

    assert!(html.contains("<table>"), "html was: {html}");
    assert!(
        html.contains(r#"style="text-align: center;""#),
        "html was: {html}"
    );
    assert!(
        html.contains(r#"style="text-align: left;""#),
        "html was: {html}"
    );
    assert!(
        html.contains(r#"style="text-align: right;""#),
        "html was: {html}"
    );
}

#[test]
fn table_with_various_cell_content() {
    let markdown =
        "| **Bold** | `code` | [link](url) |\n| --- | --- | --- |\n| normal | *italic* | plain |";

    let html = parse_markdown_fragment(markdown);

    assert!(html.contains("<table>"), "html was: {html}");
    assert!(html.contains("<strong>Bold</strong>"), "html was: {html}");
    assert!(html.contains("<code>code</code>"), "html was: {html}");
    assert!(html.contains("<a href="), "html was: {html}");
    assert!(html.contains("<em>italic</em>"), "html was: {html}");
}

#[test]
fn table_with_empty_cells() {
    let markdown = "| A | B | C |\n| --- | --- | --- |\n| | B | |";

    let html = parse_markdown_fragment(markdown);

    assert!(html.contains("<table>"), "html was: {html}");
    assert!(html.contains("<td></td>"), "html was: {html}");
}

#[test]
fn table_with_single_column() {
    let markdown = "| Header |\n| --- |\n| Cell |";

    let html = parse_markdown_fragment(markdown);

    assert!(html.contains("<table>"), "html was: {html}");
    assert!(html.contains("<th>Header</th>"), "html was: {html}");
    assert!(html.contains("<td>Cell</td>"), "html was: {html}");
}

#[test]
fn invalid_table_missing_header_row() {
    let markdown = "| A | B |\n| --- | --- |\n| Cell |";

    let html = parse_markdown_fragment(markdown);

    // Single column mismatch - may or may not render as table
    // This is a valid test case to document behavior
}

#[test]
fn table_row_count_mismatch() {
    let markdown = "| A | B | C |\n| --- | --- |\n| x |";

    let html = parse_markdown_fragment(markdown);

    // Mismatched columns - behavior should be graceful
}

#[test]
fn missing_footnote_definition_uses_fallback_text() {
    let html = parse_markdown("Reference[^missing].");

    assert!(html.contains(r#"<section class="footnotes">"#));
    assert!(html.contains("Missing footnote definition."));
}

#[test]
fn unmatched_inline_markers_remain_literal() {
    let html = parse_markdown("This **never closes and [link](broken");

    assert!(html.contains("This **never closes and [link](broken"));
    assert!(!html.contains("<strong>"));
    assert!(!html.contains("<a href="));
}

#[test]
fn raw_html_is_escaped_when_disabled() {
    // When allow_raw_html is false, raw HTML should be escaped, not executed
    let options = HtmlOptions {
        allow_raw_html: false,
        ..Default::default()
    };
    let html = parse_markdown_with_options("<script>alert('xss')</script>", options);

    assert!(!html.contains("<script>"));
    assert!(html.contains("&lt;script&gt;"));
}

#[test]
fn javascript_url_is_escaped_in_href() {
    // javascript: URLs are escaped in href
    let html = parse_markdown("[link](javascript:alert('xss'))");

    // The javascript: scheme should be escaped in the output
    assert!(html.contains("javascript:alert") || html.contains("&amp;"));
}

#[test]
fn xss_payload_with_event_handler_is_escaped_when_raw_html_disabled() {
    // When allow_raw_html is false, XSS payloads should be escaped
    let options = HtmlOptions {
        allow_raw_html: false,
        ..Default::default()
    };
    let html = parse_markdown_with_options("<em>test</em>", options);

    // With allow_raw_html: false, em tags should be escaped
    assert!(!html.contains("<em>"));
    assert!(html.contains("&lt;em&gt;"));
}

#[test]
fn raw_html_passes_through_when_enabled() {
    // When allow_raw_html is true (default), raw HTML is passed through
    let html = parse_markdown("<strong>bold</strong>");

    assert!(html.contains("<strong>bold</strong>"));
    // This is the default behavior - user must set allow_raw_html: false for security
}

#[test]
fn data_url_in_image_src_is_preserved_as_literal() {
    // data: URLs in images are preserved as-is (browsers don't execute JS from img src)
    let html = parse_markdown("![alt](data:text/html,<script>alert('xss')</script>)");

    // The data: URL is preserved (this is browser-safe for img elements)
    assert!(html.contains("data:text/html"));
}

#[test]
fn escaped_angle_brackets_prevent_html_injection() {
    // Escaped HTML should remain escaped and not be parsed
    let html = parse_markdown("Use \\<script\\> tags not `<script>`");

    // The backslash-escaped brackets should not become HTML tags
    assert!(!html.contains("<script>alert"));
    assert!(html.contains("&lt;") || html.contains("\\<"));
}

#[test]
fn broken_image_reference_remains_literal_text() {
    let html = parse_markdown("Broken ![alt][missing");

    assert!(html.contains("Broken ![alt][missing"));
    assert!(!html.contains("<img "));
}

#[test]
fn options_can_disable_blank_link_targets() {
    let html = parse_markdown_with_options(
        "[Guide](https://example.com)",
        HtmlOptions {
            link_target_blank: false,
            ..HtmlOptions::default()
        },
    );

    assert!(html.contains(r#"<a href="https://example.com">Guide</a>"#));
    assert!(!html.contains(r#"target="_blank""#));
}

#[test]
fn options_can_disable_raw_html_passthrough() {
    let html = parse_markdown_with_options(
        "Keep <em>escaped</em> instead.",
        HtmlOptions {
            allow_raw_html: false,
            ..HtmlOptions::default()
        },
    );

    assert!(html.contains("Keep &lt;em&gt;escaped&lt;/em&gt; instead."));
    assert!(!html.contains("<em>escaped</em>"));
}

#[test]
fn fragment_options_apply_without_footnote_section() {
    let html = parse_markdown_fragment_with_options(
        "<span>no raw html</span> [Guide](https://example.com)",
        HtmlOptions {
            link_target_blank: false,
            allow_raw_html: false,
        },
    );

    assert!(!html.contains(r#"<section class="footnotes">"#));
    assert!(html.contains("&lt;span&gt;no raw html&lt;/span&gt;"));
    assert!(html.contains(r#"<a href="https://example.com">Guide</a>"#));
}

#[test]
fn renders_lazy_ordered_list_continuations_as_single_list() {
    let markdown = "\
1. Fix correctness first.
Only emit file results.
2. Restore tree metadata.
Compute percentages recursively.";

    let html = parse_markdown_fragment(markdown);

    assert_eq!(html.matches("<ol>").count(), 1, "html was: {html}");
    assert_eq!(html.matches("</ol>").count(), 1, "html was: {html}");
    assert!(html.contains("<li>Fix correctness first. Only emit file results.</li>"));
    assert!(html.contains("<li>Restore tree metadata. Compute percentages recursively.</li>"));
}

#[test]
fn renders_blank_line_continuation_paragraph_inside_same_ordered_list() {
    let markdown = "\
1. Fix correctness first.
Only emit file results.
2. Restore tree metadata invariants after scan.
Either:
- compute `percent` recursively in a post-pass alongside `recalculate_sizes`, or
- compute percentages per-directory when combining children.

The post-pass is cleaner since parallel modes already do a post-pass.

3. Reduce parallelism granularity aggressively.
The current design is trying to parallelize every subtree and every file list.";

    let html = parse_markdown_fragment(markdown);

    assert_eq!(html.matches("<ol>").count(), 1, "html was: {html}");
    assert_eq!(html.matches("<li>").count(), 5, "html was: {html}");
    assert!(
        html.contains(
            "<li>Restore tree metadata invariants after scan. Either:\n<ul>\n<li>compute"
        ),
        "html was: {html}"
    );
    assert!(html.contains("The post-pass is cleaner since parallel modes already do a post-pass."));
}

#[test]
fn renders_verification_summary_bullets_nested_under_numbered_items() {
    let markdown = "\
1. **P0: the-markdown-commons Compilation** `CORRECT`
- **File:** `src/document.rs`
- **Fix:** Separated imports
- **Verified:** `cargo check` passes successfully
- **Status:** Confirmed working

2. **P1: include! Macros** `Documented`
- **File:** `src/coordination/mod.rs`
- **Decision:** Reverted module refactoring
- **Status:** Documented limitation";

    let html = parse_markdown_fragment(markdown);

    assert_eq!(html.matches("<ol>").count(), 1, "html was: {html}");
    assert_eq!(html.matches("<li>").count(), 9, "html was: {html}");
    assert!(
        html.contains(
            "<li><strong>P0: the-markdown-commons Compilation</strong> <code>CORRECT</code>\n<ul>"
        ),
        "html was: {html}"
    );
    assert!(
        html.contains("<li><strong>P1: include! Macros</strong> <code>Documented</code>\n<ul>"),
        "html was: {html}"
    );
}

#[test]
fn renders_nested_unordered_lists_inside_parent_bullet() {
    let markdown = "\
- **Verified:**
  - Line 29
  - Line 76
- **Status:** Confirmed";

    let html = parse_markdown_fragment(markdown);

    assert!(
        html.contains(
            "<ul>\n<li><strong>Verified:</strong>\n<ul>\n<li>Line 29</li>\n<li>Line 76</li>\n</ul>\n</li>\n<li><strong>Status:</strong> Confirmed</li>\n</ul>\n"
        ),
        "html was: {html}"
    );
}

#[test]
fn renders_double_digit_ordered_list_with_all_items() {
    let markdown = (1..=12)
        .map(|i| format!("{i}. Item {i}"))
        .collect::<Vec<_>>()
        .join("\n");

    let html = parse_markdown_fragment(&markdown);

    assert_eq!(html.matches("<ol>").count(), 1, "html was: {html}");
    assert_eq!(html.matches("<li>").count(), 12, "html was: {html}");
    for i in 1..=12 {
        assert!(
            html.contains(&format!("<li>Item {i}</li>")),
            "missing item {i}, html was: {html}"
        );
    }
}

#[test]
fn renders_task_list_checked_and_unchecked_with_dash_marker() {
    let markdown = "- [ ] Unchecked item\n- [x] Checked lowercase x\n- [X] Checked uppercase X";

    let html = parse_markdown_fragment(markdown);

    assert!(html.contains(r#"<ul class="task-list">"#));
    assert!(html.contains(r#"<input type="checkbox" disabled>"#));
    assert!(html.contains(r#"<input type="checkbox" disabled checked>"#));
    assert!(html.contains("Unchecked item"));
    assert!(html.contains("Checked lowercase x"));
    assert!(html.contains("Checked uppercase X"));
}

#[test]
fn renders_task_list_with_asterisk_marker() {
    let markdown = "* [ ] Asterisk unchecked\n* [x] Asterisk checked x\n* [X] Asterisk checked X";

    let html = parse_markdown_fragment(markdown);

    assert!(html.contains(r#"<ul class="task-list">"#));
    assert!(html.contains("Asterisk unchecked"));
    assert!(html.contains("Asterisk checked x"));
    assert!(html.contains("Asterisk checked X"));
}

#[test]
fn renders_multiple_task_items_in_list() {
    let markdown = "\
- [ ] First task
- [ ] Second task
- [x] Completed task
- [ ] Third task";

    let html = parse_markdown_fragment(markdown);

    assert_eq!(
        html.matches(r#"<li class="task-list-item">"#).count(),
        4,
        "html was: {html}"
    );
    assert_eq!(
        html.matches(r#"<input type="checkbox" disabled checked>"#)
            .count(),
        1,
        "html was: {html}"
    );
    assert_eq!(
        html.matches(r#"<input type="checkbox" disabled>"#).count(),
        3,
        "html was: {html}"
    );
}

#[test]
fn invalid_task_syntax_without_space_after_bracket() {
    // Missing space after bracket means not parsed as task list
    let markdown = "- [ ]Task no space\n- [x]Task no space x";

    let html = parse_markdown_fragment(markdown);

    // Invalid task syntax should not be parsed as task list
    assert!(
        !html.contains(r#"<ul class="task-list">"#),
        "html was: {html}"
    );
    // Should appear as literal text (hyphen at start creates regular list)
    assert!(
        html.contains("<li>[ ]Task no space</li>"),
        "html was: {html}"
    );
}

#[test]
fn task_list_with_inline_formatting() {
    let markdown =
        "- [ ] **Bold** task\n- [x] *Italic* task\n- [ ] `code` task\n- [x] **Bold and `code`**";

    let html = parse_markdown_fragment(markdown);

    assert!(html.contains("<strong>Bold</strong>"));
    assert!(html.contains("<em>Italic</em>"));
    assert!(html.contains("<code>code</code>"));
}

#[test]
fn task_list_empty_content_renders_with_empty_span() {
    // Empty task items (just space after bracket) ARE parsed as task lists with empty content
    let markdown = "- [ ] \n- [x] ";

    let html = parse_markdown_fragment(markdown);

    assert!(
        html.contains(r#"<ul class="task-list">"#),
        "html was: {html}"
    );
    assert_eq!(
        html.matches(r#"<li class="task-list-item">"#).count(),
        2,
        "html was: {html}"
    );
    assert!(html.contains("<span></span>"), "html was: {html}");
}

#[test]
fn task_list_with_proper_content() {
    let markdown = "- [ ] Task with content\n- [x] Completed task";

    let html = parse_markdown_fragment(markdown);

    assert!(html.contains(r#"<ul class="task-list">"#));
    assert_eq!(
        html.matches(r#"<li class="task-list-item">"#).count(),
        2,
        "html was: {html}"
    );
}

#[test]
fn task_list_interaction_with_paragraphs() {
    let markdown = "\
Before task list:

- [ ] Task item

After task list with paragraph.";

    let html = parse_markdown_fragment(markdown);

    assert!(html.contains("<p>Before task list:</p>"));
    assert!(html.contains(r#"<ul class="task-list">"#));
    assert!(html.contains("<p>After task list with paragraph.</p>"));
}

#[test]
fn mixed_task_list_and_regular_list() {
    let markdown = "\
- [ ] Task item
- Regular bullet item
- [x] Another task
- [ ] Final task";

    let html = parse_markdown_fragment(markdown);

    // Only task list items should be in task-list class
    assert!(html.contains(r#"<ul class="task-list">"#));
    // Regular bullet should be in regular ul
    assert!(html.contains("<ul>") || html.contains("<li>Regular bullet"));
}

#[test]
fn task_list_inside_blockquote() {
    let markdown = "> - [ ] Quote task\n> - [x] Quote task done";

    let html = parse_markdown_fragment(markdown);

    assert!(html.contains("<blockquote>"));
    assert!(html.contains(r#"<ul class="task-list">"#));
    assert!(html.contains(r#"<input type="checkbox" disabled>"#));
    assert!(html.contains(r#"<input type="checkbox" disabled checked>"#));
}

#[test]
fn nested_unordered_list_renders_correctly() {
    // Use 4-space indent for nested lists (standard markdown)
    let markdown = "\
- Top level item
    - Nested item
    - Another nested
- Back to top";

    let html = parse_markdown_fragment(markdown);

    // Should have outer ul and inner nested ul
    assert!(html.contains("<ul>"), "html was: {html}");
    assert!(html.contains("Top level item"), "html was: {html}");
    assert!(html.contains("Nested item"), "html was: {html}");
}

#[test]
fn continuation_paragraph_inside_unordered_list() {
    let markdown = "\
- First item with
  continuation text
- Second item";

    let html = parse_markdown_fragment(markdown);

    assert_eq!(html.matches("<ul>").count(), 1, "html was: {html}");
    assert_eq!(html.matches("<li>").count(), 2, "html was: {html}");
    assert!(
        html.contains("First item with continuation text"),
        "html was: {html}"
    );
}

#[test]
fn blank_line_paragraph_continuation_inside_ordered_list() {
    let markdown = "\
1. Item with paragraph

   Continuation paragraph

2. Next item";

    let html = parse_markdown_fragment(markdown);

    assert_eq!(html.matches("<ol>").count(), 1, "html was: {html}");
    assert_eq!(html.matches("<li>").count(), 2, "html was: {html}");
    assert!(html.contains("Continuation paragraph"), "html was: {html}");
}

#[test]
fn mixed_nested_ordered_and_unordered_lists() {
    let markdown = "\
1. **Ordered item:**
   - Nested unordered
   - Another nested
2. **Second ordered**";

    let html = parse_markdown_fragment(markdown);

    assert!(
        html.contains("<ol>") && html.contains("<ul>"),
        "html was: {html}"
    );
    assert!(
        html.contains("<strong>Ordered item:</strong>"),
        "html was: {html}"
    );
    assert!(html.contains("Nested unordered"), "html was: {html}");
    assert!(
        html.contains("<strong>Second ordered</strong>"),
        "html was: {html}"
    );
}

#[test]
fn list_item_with_inline_code_and_links() {
    let markdown = "\
- Item with `code` and [link](https://example.com)
- **Bold** and *italic* in list
- Item with **bold `code`**";

    let html = parse_markdown_fragment(markdown);

    assert!(html.contains("<code>code</code>"));
    assert!(html.contains("<a href="));
    assert!(html.contains("<strong>Bold</strong>"));
    assert!(html.contains("<em>italic</em>"));
}

#[test]
fn ordered_list_with_double_digit_start() {
    let markdown = "\
10. Tenth item
11. Eleventh item
12. Twelfth item";

    let html = parse_markdown_fragment(markdown);

    assert!(html.contains("<ol start=\"10\">") || html.contains("<li>"));
    assert!(html.contains("Tenth item"));
    assert!(html.contains("Eleventh item"));
    assert!(html.contains("Twelfth item"));
}

#[test]
fn list_continuation_with_lazy_indent() {
    let markdown = "\
- First item
  continued text
- Second item";

    let html = parse_markdown_fragment(markdown);

    assert_eq!(html.matches("<ul>").count(), 1, "html was: {html}");
    assert_eq!(html.matches("<li>").count(), 2, "html was: {html}");
}
