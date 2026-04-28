//! Repo-owned markdown renderer with improved table support.
//!
//! This module avoids `egui_commonmark` entirely. It renders markdown blocks
//! with explicit egui layout so message rendering stays deterministic.
//!
//! Extracted from `rust-chatbot/src/markdown.rs`.

#[cfg(test)]
mod headless_layout_tests;
#[cfg(test)]
mod headless_tests;
mod inline;
mod lists;
#[cfg(test)]
mod parse;
mod render;
mod segments;
mod style;

#[allow(unused_imports)]
use crate::common::{self, Alignment};

pub use render::{render_markdown, render_markdown_with_options, MarkdownRenderOptions};
pub use segments::parse_markdown_segments;

/// Cache for egui markdown rendering.
///
/// Contains a `DocumentCache` for parsed documents and any egui-specific
/// rendering state. Reusing the same cache across renders avoids re-parsing
/// the same markdown content.
///
/// # Example
///
/// ```
/// use tmc::{egui::MarkdownCache, render_markdown};
/// use egui::Ui;
///
/// fn render(ui: &mut Ui, cache: &mut MarkdownCache, content: &str) {
///     render_markdown(ui, cache, content, "my-id");
/// }
/// ```
#[derive(Debug, Default)]
pub struct MarkdownCache {
    /// The underlying document cache for parsed markdown.
    pub document_cache: crate::DocumentCache,
}

/// A table extracted from markdown for custom rendering.
///
/// `MarkdownTable` holds the raw string content of table cells without
/// inline formatting applied. Use `parse_markdown_segments` to extract
/// tables from markdown content.
#[derive(Debug, Clone)]
pub struct MarkdownTable {
    /// Table header column names.
    pub headers: Vec<String>,
    /// Column alignment hints (currently unused in egui rendering).
    #[allow(dead_code)] // Reserved for future column alignment support
    pub alignments: Vec<crate::common::Alignment>,
    /// Table data rows.
    pub rows: Vec<Vec<String>>,
}

/// Segments produced by parsing markdown content.
///
/// `MarkdownSegment` represents either plain text or a table extracted
/// from markdown. Use `parse_markdown_segments` to split mixed content.
#[derive(Debug)]
pub enum MarkdownSegment {
    /// Plain text content with inline formatting.
    Text(String),
    /// A parsed table.
    Table(MarkdownTable),
}

#[cfg(test)]
use inline::{parse_inline_formatting, InlineSpan};
#[cfg(test)]
use parse::{parse_fence_start, parse_heading, parse_list_marker, parse_setext_heading};
#[cfg(test)]
use render::count_emoji;
#[cfg(test)]
use segments::{is_separator_row, is_table_row, parse_alignments, parse_table, parse_table_row};

#[cfg(test)]
mod tests {
    use super::*;

    fn headless_input(time: f64) -> egui::RawInput {
        headless_input_with_size(time, egui::vec2(800.0, 600.0))
    }

    fn headless_input_with_size(time: f64, size: egui::Vec2) -> egui::RawInput {
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, size)),
            time: Some(time),
            ..Default::default()
        }
    }

    fn run_markdown_frame(
        ctx: &egui::Context,
        input: egui::RawInput,
        cache: &mut MarkdownCache,
        content: &str,
        id_salt: &str,
    ) -> egui::FullOutput {
        ctx.run(input, |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                render_markdown(ui, cache, content, id_salt);
            });
        })
    }

    fn run_markdown_frame_with_options(
        ctx: &egui::Context,
        input: egui::RawInput,
        cache: &mut MarkdownCache,
        content: &str,
        id_salt: &str,
        options: &MarkdownRenderOptions<'_>,
    ) -> egui::FullOutput {
        ctx.run(input, |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                render_markdown_with_options(ui, cache, content, id_salt, options);
            });
        })
    }

    fn render_markdown_painted_bounds(
        screen_size: egui::Vec2,
        content: &str,
        id_salt: &str,
    ) -> egui::Rect {
        let ctx = egui::Context::default();
        let mut cache = MarkdownCache::default();
        let input = headless_input_with_size(0.0, screen_size);

        let output = ctx.run(input, |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                render_markdown(ui, &mut cache, content, id_salt);
            });
        });
        let primitives = ctx.tessellate(output.shapes, output.pixels_per_point);
        let mut bounds = egui::Rect::NOTHING;

        for clipped in primitives {
            if let egui::epaint::Primitive::Mesh(mesh) = clipped.primitive {
                for vertex in mesh.vertices {
                    if bounds.is_positive() {
                        bounds.extend_with(vertex.pos);
                    } else {
                        bounds = egui::Rect::from_min_max(vertex.pos, vertex.pos);
                    }
                }
            }
        }

        bounds
    }

    fn count_shape_kinds(shapes: &[egui::epaint::ClippedShape]) -> (usize, usize, usize) {
        fn walk(shape: &egui::epaint::Shape, counts: &mut (usize, usize, usize)) {
            match shape {
                egui::epaint::Shape::Vec(shapes) => {
                    for shape in shapes {
                        walk(shape, counts);
                    }
                }
                egui::epaint::Shape::Text(_) => counts.0 += 1,
                egui::epaint::Shape::LineSegment { .. } => counts.1 += 1,
                egui::epaint::Shape::Rect(_) => counts.2 += 1,
                _ => {}
            }
        }

        let mut counts = (0, 0, 0);
        for clipped in shapes {
            walk(&clipped.shape, &mut counts);
        }
        counts
    }

    #[derive(Debug)]
    struct RenderSummary {
        text_shapes: usize,
        line_segments: usize,
        rect_shapes: usize,
        width: f32,
    }

    fn render_markdown_summary(
        screen_size: egui::Vec2,
        content: &str,
        id_salt: &str,
    ) -> RenderSummary {
        let ctx = egui::Context::default();
        let mut cache = MarkdownCache::default();
        let output = run_markdown_frame(
            &ctx,
            headless_input_with_size(0.0, screen_size),
            &mut cache,
            content,
            id_salt,
        );
        let (text_shapes, line_segments, rect_shapes) = count_shape_kinds(&output.shapes);
        let bounds = render_markdown_painted_bounds(screen_size, content, id_salt);

        RenderSummary {
            text_shapes,
            line_segments,
            rect_shapes,
            width: bounds.width(),
        }
    }

    #[test]
    fn test_inline_code_parsing() {
        let segments = parse_inline_formatting("Use `foo` and `bar` for testing");

        assert_eq!(segments.len(), 5);
        assert_eq!(
            segments[0],
            InlineSpan {
                text: "Use ".to_string(),
                is_code: false,
                is_bold: false,
            }
        );
        assert_eq!(
            segments[1],
            InlineSpan {
                text: "foo".to_string(),
                is_code: true,
                is_bold: false,
            }
        );
        assert_eq!(
            segments[2],
            InlineSpan {
                text: " and ".to_string(),
                is_code: false,
                is_bold: false,
            }
        );
        assert_eq!(
            segments[3],
            InlineSpan {
                text: "bar".to_string(),
                is_code: true,
                is_bold: false,
            }
        );
        assert_eq!(
            segments[4],
            InlineSpan {
                text: " for testing".to_string(),
                is_code: false,
                is_bold: false,
            }
        );
    }

    #[test]
    fn test_inline_bold_parsing() {
        let segments = parse_inline_formatting("This is **bold** text");

        assert_eq!(segments.len(), 3);
        assert_eq!(
            segments[0],
            InlineSpan {
                text: "This is ".to_string(),
                is_code: false,
                is_bold: false,
            }
        );
        assert_eq!(
            segments[1],
            InlineSpan {
                text: "bold".to_string(),
                is_code: false,
                is_bold: true,
            }
        );
        assert_eq!(
            segments[2],
            InlineSpan {
                text: " text".to_string(),
                is_code: false,
                is_bold: false,
            }
        );
    }

    #[test]
    fn test_inline_mixed_formatting() {
        let segments = parse_inline_formatting("Use `code` and **bold** together");

        assert_eq!(segments.len(), 5);
        assert_eq!(
            segments[0],
            InlineSpan {
                text: "Use ".to_string(),
                is_code: false,
                is_bold: false,
            }
        );
        assert_eq!(
            segments[1],
            InlineSpan {
                text: "code".to_string(),
                is_code: true,
                is_bold: false,
            }
        );
        assert_eq!(
            segments[2],
            InlineSpan {
                text: " and ".to_string(),
                is_code: false,
                is_bold: false,
            }
        );
        assert_eq!(
            segments[3],
            InlineSpan {
                text: "bold".to_string(),
                is_code: false,
                is_bold: true,
            }
        );
        assert_eq!(
            segments[4],
            InlineSpan {
                text: " together".to_string(),
                is_code: false,
                is_bold: false,
            }
        );
    }

    #[test]
    fn test_inline_bold_code_parsing() {
        let segments = parse_inline_formatting("**`format: 'raw'`**");

        assert_eq!(segments.len(), 1);
        assert_eq!(
            segments[0],
            InlineSpan {
                text: "format: 'raw'".to_string(),
                is_code: true,
                is_bold: true,
            }
        );
    }

    #[test]
    fn test_count_emoji() {
        assert_eq!(count_emoji("✅ Verified"), 1);
        assert_eq!(count_emoji("❌ Not tested"), 1);
        assert_eq!(count_emoji("No emoji here"), 0);
        assert_eq!(count_emoji("✅❌ both"), 2);
    }

    #[test]
    fn test_emoji_table() {
        let content = r#"### Side-by-Side Comparison
| Aspect | Current Tests | Expected Tests |
|--------|---------------|----------------|
| Mock handler works | ✅ Verified | ✅ Verified |
| URL pattern matching | ✅ Verified | ✅ Verified |
| Video ID extraction | ✅ Verified | ✅ Verified |
| Extractor parses HTML | ❌ Not tested | ✅ Verified |
| Extractor parses API responses | ❌ Not tested | ✅ Verified |
| Metadata extraction (title, etc.) | ❌ Not tested | ✅ Verified |
| Format list extraction | ❌ Not tested | ✅ Verified |
| Error handling during extraction | ❌ Not tested | ✅ Verified |"#;

        let segments = parse_markdown_segments(content);

        // Should have text (header) and table
        println!("Number of segments: {}", segments.len());
        for (i, seg) in segments.iter().enumerate() {
            match seg {
                MarkdownSegment::Text(t) => println!("Segment {}: Text = {:?}", i, t),
                MarkdownSegment::Table(t) => {
                    println!(
                        "Segment {}: Table with {} headers, {} rows",
                        i,
                        t.headers.len(),
                        t.rows.len()
                    );
                    println!("  Headers: {:?}", t.headers);
                    for (j, row) in t.rows.iter().enumerate() {
                        println!("  Row {}: {:?}", j, row);
                    }
                }
            }
        }

        assert_eq!(segments.len(), 2, "Should have text and table segments");

        match &segments[1] {
            MarkdownSegment::Table(table) => {
                assert_eq!(table.headers.len(), 3, "Should have 3 headers");
                assert_eq!(table.rows.len(), 8, "Should have 8 data rows");

                // Check first row content
                assert_eq!(table.rows[0][0], "Mock handler works");
                assert_eq!(table.rows[0][1], "✅ Verified");
                assert_eq!(table.rows[0][2], "✅ Verified");
            }
            _ => panic!("Expected table segment"),
        }
    }

    #[test]
    fn test_is_table_row() {
        assert!(is_table_row("| a | b |"));
        assert!(is_table_row("|---|---|"));
        assert!(is_table_row("| test | value |"));
        assert!(!is_table_row("regular text"));
        assert!(!is_table_row(""));
    }

    #[test]
    fn test_is_separator_row() {
        assert!(is_separator_row("|---|---|"));
        assert!(is_separator_row("| --- | --- |"));
        assert!(is_separator_row("|:---|---:|"));
        assert!(is_separator_row("|:---:|:---:|"));
        assert!(!is_separator_row("| a | b |"));
    }

    #[test]
    fn test_parse_table_row() {
        let row = parse_table_row("| a | b | c |");
        assert_eq!(row, vec!["a", "b", "c"]);

        let row2 = parse_table_row("| hello | world |");
        assert_eq!(row2, vec!["hello", "world"]);
    }

    #[test]
    fn test_parse_alignments() {
        let aligns = parse_alignments("|:---|:---:|---:|", 3);
        assert_eq!(
            aligns,
            vec![Alignment::Left, Alignment::Center, Alignment::Right]
        );
    }

    #[test]
    fn test_parse_markdown_segments() {
        let content = r#"Some text before

| Header 1 | Header 2 |
|----------|----------|
| Cell 1   | Cell 2   |

Some text after"#;

        let segments = parse_markdown_segments(content);
        assert_eq!(segments.len(), 3);

        match &segments[0] {
            MarkdownSegment::Text(_) => {}
            _ => panic!("Expected text segment"),
        }

        match &segments[1] {
            MarkdownSegment::Table(table) => {
                assert_eq!(table.headers.len(), 2);
                assert_eq!(table.rows.len(), 1);
            }
            _ => panic!("Expected table segment"),
        }

        match &segments[2] {
            MarkdownSegment::Text(_) => {}
            _ => panic!("Expected text segment"),
        }
    }

    #[test]
    fn test_parse_markdown_segments_ignores_tables_inside_fenced_code() {
        let content = r#"```md
| not | a table |
| --- | ------- |
```

| real | table |
| --- | --- |
| yes | true |"#;

        let segments = parse_markdown_segments(content);

        assert_eq!(segments.len(), 2);
        assert!(
            matches!(&segments[0], MarkdownSegment::Text(text) if text.contains("| not | a table |"))
        );
        assert!(matches!(&segments[1], MarkdownSegment::Table(table) if table.rows.len() == 1));
    }

    #[test]
    fn test_parse_table_pads_and_truncates_rows() {
        let lines = [
            "| A | B |",
            "| --- | --- |",
            "| one |",
            "| two | extra | ignored |",
        ];

        let table = parse_table(&lines).expect("table should parse");

        assert_eq!(table.headers, vec!["A", "B"]);
        assert_eq!(table.rows[0], vec!["one", ""]);
        assert_eq!(table.rows[1], vec!["two", "extra"]);
    }

    #[test]
    fn test_invalid_table_falls_back_to_text() {
        let content = "| missing separator |\n| still text |";

        let segments = parse_markdown_segments(content);

        assert_eq!(segments.len(), 1);
        assert!(
            matches!(&segments[0], MarkdownSegment::Text(text) if text.contains("| missing separator |"))
        );
    }

    #[test]
    fn test_normalize_fenced_code_languages() {
        let input = "```Rust\nfn main() {}\n```\n\n```Shell session\nls\n```";

        let normalized = common::normalize_fenced_code_languages(input);

        assert!(normalized.contains("```rs\nfn main() {}\n```"));
        assert!(normalized.contains("```sh session\nls\n```"));
    }

    #[test]
    fn test_parse_heading_variants() {
        assert_eq!(parse_heading("# Title"), Some((1, "Title")));
        assert_eq!(parse_heading("### Deep"), Some((3, "Deep")));
        assert_eq!(parse_heading("####### Too deep"), None);
    }

    #[test]
    fn test_parse_setext_heading_variants() {
        let lines = ["Title", "=====", "", "Subtitle", "-----"];

        assert_eq!(parse_setext_heading(&lines, 0), Some(1));
        assert_eq!(parse_setext_heading(&lines, 3), Some(2));
    }

    #[test]
    fn test_parse_list_marker_variants() {
        assert_eq!(
            parse_list_marker("- item"),
            Some((0, "-".to_string(), "item"))
        );
        assert_eq!(
            parse_list_marker("    * nested"),
            Some((1, "*".to_string(), "nested"))
        );
        assert_eq!(
            parse_list_marker("12. ordered"),
            Some((0, "12.".to_string(), "ordered"))
        );
    }

    #[test]
    fn test_parse_fence_start_normalizes_aliases() {
        assert_eq!(
            parse_fence_start("```typescript"),
            Some(('`', Some("ts".to_string())))
        );
        assert_eq!(
            parse_fence_start("~~~Python"),
            Some(('~', Some("py".to_string())))
        );
        assert_eq!(parse_fence_start("```"), Some(('`', None)));
    }

    #[test]
    fn test_headless_render_markdown_produces_shapes() {
        let ctx = egui::Context::default();
        let mut cache = MarkdownCache::default();
        let content = r#"# Title

Paragraph with **bold** and [link](https://example.com).

> Quote

| A | B |
| --- | --- |
| 1 | 2 |

```rs
fn main() {}
```"#;

        let output = run_markdown_frame(
            &ctx,
            headless_input(0.0),
            &mut cache,
            content,
            "headless_shapes",
        );

        assert!(!output.shapes.is_empty());
        assert!(output.platform_output.open_url.is_none());
        assert!(output.platform_output.copied_text.is_empty());
    }

    #[test]
    fn test_headless_hyperlink_click_opens_url() {
        let ctx = egui::Context::default();
        let mut cache = MarkdownCache::default();
        let content = "[example](https://example.com)";

        let _ = run_markdown_frame(
            &ctx,
            headless_input(0.0),
            &mut cache,
            content,
            "headless_link",
        );

        let click_pos = egui::pos2(40.0, 20.0);
        let mut input = headless_input(1.0);
        input.events = vec![
            egui::Event::PointerMoved(click_pos),
            egui::Event::PointerButton {
                pos: click_pos,
                button: egui::PointerButton::Primary,
                pressed: true,
                modifiers: Default::default(),
            },
            egui::Event::PointerButton {
                pos: click_pos,
                button: egui::PointerButton::Primary,
                pressed: false,
                modifiers: Default::default(),
            },
        ];

        let output = run_markdown_frame(&ctx, input, &mut cache, content, "headless_link");

        assert_eq!(
            output
                .platform_output
                .open_url
                .as_ref()
                .map(|open| open.url.as_str()),
            Some("https://example.com")
        );
    }

    #[test]
    fn test_headless_hyperlink_renders_as_selectable_label() {
        let ctx = egui::Context::default();
        let mut cache = MarkdownCache::default();
        let content = "[example link](https://example.com)";

        // First frame: layout the markdown.
        let _ = run_markdown_frame(
            &ctx,
            headless_input(0.0),
            &mut cache,
            content,
            "headless_link_selectable",
        );

        // Second frame: render again and verify shapes are produced (the link
        // text is rendered via a selectable Label so it generates text shapes).
        let output = run_markdown_frame(
            &ctx,
            headless_input(1.0),
            &mut cache,
            content,
            "headless_link_selectable",
        );

        let (text_shapes, _line_segments, _rect_shapes) = count_shape_kinds(&output.shapes);
        assert!(text_shapes > 0, "expected text shapes for the link label");
    }

    #[test]
    fn test_headless_hovered_link_keeps_underline_above_code_on_next_line() {
        let ctx = egui::Context::default();
        let mut cache = MarkdownCache::default();
        let content = "[example](https://example.com)\n`code`";

        let _ = run_markdown_frame(
            &ctx,
            headless_input(0.0),
            &mut cache,
            content,
            "headless_link_hover_code_below",
        );

        let hover_pos = egui::pos2(40.0, 20.0);
        let mut input = headless_input(1.0);
        input.events = vec![egui::Event::PointerMoved(hover_pos)];

        let output = run_markdown_frame(
            &ctx,
            input,
            &mut cache,
            content,
            "headless_link_hover_code_below",
        );
        let (_text_shapes, line_segments, _rect_shapes) = count_shape_kinds(&output.shapes);

        assert!(line_segments > 0);
    }

    #[test]
    fn test_headless_code_block_copy_button_copies_text() {
        let ctx = egui::Context::default();
        let mut cache = MarkdownCache::default();
        let content = "```rs\nfn main() {}\n```";

        let _ = run_markdown_frame(
            &ctx,
            headless_input(0.0),
            &mut cache,
            content,
            "headless_copy",
        );

        let click_pos = egui::pos2(760.0, 24.0);
        let mut input = headless_input(1.0);
        input.events = vec![
            egui::Event::PointerMoved(click_pos),
            egui::Event::PointerButton {
                pos: click_pos,
                button: egui::PointerButton::Primary,
                pressed: true,
                modifiers: Default::default(),
            },
            egui::Event::PointerButton {
                pos: click_pos,
                button: egui::PointerButton::Primary,
                pressed: false,
                modifiers: Default::default(),
            },
        ];

        let output = run_markdown_frame(&ctx, input, &mut cache, content, "headless_copy");

        assert_eq!(output.platform_output.copied_text, "fn main() {}");
    }

    #[test]
    fn test_headless_blockquote_renders_quote_bar() {
        let ctx = egui::Context::default();
        let mut cache = MarkdownCache::default();
        let content = "> quoted\n>\n> second line";

        let output = run_markdown_frame(
            &ctx,
            headless_input(0.0),
            &mut cache,
            content,
            "headless_blockquote",
        );
        let (text_shapes, line_segments, _rects) = count_shape_kinds(&output.shapes);

        assert!(text_shapes > 0);
        assert!(line_segments > 0);
    }

    #[test]
    fn test_softbreak_paragraph_does_not_add_extra_vertical_spacing() {
        let multiline = "Current live state on `edge-node-01.example`:\n`pingline-first-boot-install.service`: `activating`\n`pingline-viewer.service`: `inactive`\nPatch `8b2` is definitely active now";
        let collapsed = "Current live state on `edge-node-01.example`: `pingline-first-boot-install.service`: `activating` `pingline-viewer.service`: `inactive` Patch `8b2` is definitely active now";

        let multiline_rect = render_markdown_painted_bounds(
            egui::vec2(2200.0, 400.0),
            multiline,
            "headless_softbreak_multiline",
        );
        let collapsed_rect = render_markdown_painted_bounds(
            egui::vec2(2200.0, 400.0),
            collapsed,
            "headless_softbreak_collapsed",
        );

        assert!(multiline_rect.height() <= collapsed_rect.height() + 6.0);
    }
}
