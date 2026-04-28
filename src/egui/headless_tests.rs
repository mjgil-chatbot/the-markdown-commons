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

    let _ = run_markdown_frame(
        &ctx,
        headless_input_with_size(0.0, screen_size),
        &mut cache,
        content,
        id_salt,
    );
    let output = run_markdown_frame(
        &ctx,
        headless_input_with_size(1.0, screen_size),
        &mut cache,
        content,
        id_salt,
    );

    output
        .shapes
        .iter()
        .filter_map(|clipped| {
            let visible = clipped
                .shape
                .visual_bounding_rect()
                .intersect(clipped.clip_rect);
            if visible.is_positive() {
                Some(visible)
            } else {
                None
            }
        })
        .reduce(|acc, rect| acc.union(rect))
        .unwrap_or_else(|| egui::Rect::from_min_size(egui::Pos2::ZERO, egui::Vec2::ZERO))
}

fn count_shape_kinds(shapes: &[egui::epaint::ClippedShape]) -> (usize, usize, usize) {
    let mut counts = (0, 0, 0);
    for clipped in shapes {
        match &clipped.shape {
            egui::epaint::Shape::Text(_) => counts.0 += 1,
            egui::epaint::Shape::LineSegment { .. } => counts.1 += 1,
            egui::epaint::Shape::Rect(_) => counts.2 += 1,
            _ => {}
        }
    }
    counts
}

#[derive(Debug, Default, PartialEq, Eq)]
struct MarkdownRenderSummary {
    text_shapes: usize,
    line_segments: usize,
    rect_shapes: usize,
    width: i32,
}

fn render_markdown_summary(
    screen_size: egui::Vec2,
    content: &str,
    id_salt: &str,
) -> MarkdownRenderSummary {
    let ctx = egui::Context::default();
    let mut cache = MarkdownCache::default();

    let _ = run_markdown_frame(
        &ctx,
        headless_input_with_size(0.0, screen_size),
        &mut cache,
        content,
        id_salt,
    );
    let output = run_markdown_frame(
        &ctx,
        headless_input_with_size(1.0, screen_size),
        &mut cache,
        content,
        id_salt,
    );
    let bounds = render_markdown_painted_bounds(screen_size, content, id_salt);
    let (text_shapes, line_segments, rect_shapes) = count_shape_kinds(&output.shapes);

    MarkdownRenderSummary {
        text_shapes,
        line_segments,
        rect_shapes,
        width: bounds.width().round() as i32,
    }
}

#[test]
fn untagged_code_block_renders_text_shapes() {
    let ctx = egui::Context::default();
    let mut cache = MarkdownCache::default();
    let content = "```\ndocker compose up -d  # start\n```";

    let _ = run_markdown_frame(
        &ctx,
        headless_input(0.0),
        &mut cache,
        content,
        "untagged_code",
    );
    let output = run_markdown_frame(
        &ctx,
        headless_input(1.0),
        &mut cache,
        content,
        "untagged_code",
    );

    let (text_shapes, _line_segments, _rect_shapes) = count_shape_kinds(&output.shapes);
    assert!(
        text_shapes > 0,
        "expected text shapes for untagged code block"
    );
}

#[test]
fn code_language_override_remaps_tag() {
    let ctx = egui::Context::default();
    let mut cache_with = MarkdownCache::default();
    let mut cache_without = MarkdownCache::default();
    let content = "```text\necho hello  # comment\n```";

    let options_with = MarkdownRenderOptions {
        default_code_language: None,
        code_language_overrides: &[("text", "sh")],
    };
    let options_without = MarkdownRenderOptions::default();

    let _ = run_markdown_frame_with_options(
        &ctx,
        headless_input(0.0),
        &mut cache_with,
        content,
        "override_with",
        &options_with,
    );
    let _ = run_markdown_frame_with_options(
        &ctx,
        headless_input(0.0),
        &mut cache_without,
        content,
        "override_without",
        &options_without,
    );

    let output_with = run_markdown_frame_with_options(
        &ctx,
        headless_input(1.0),
        &mut cache_with,
        content,
        "override_with",
        &options_with,
    );
    let output_without = run_markdown_frame_with_options(
        &ctx,
        headless_input(1.0),
        &mut cache_without,
        content,
        "override_without",
        &options_without,
    );

    let (text_with, _, _) = count_shape_kinds(&output_with.shapes);
    let (text_without, _, _) = count_shape_kinds(&output_without.shapes);
    assert!(text_with > 0, "override render should produce text shapes");
    assert!(
        text_without > 0,
        "default render should produce text shapes"
    );
}

#[test]
fn default_code_language_does_not_override_explicit_tag() {
    let ctx = egui::Context::default();
    let mut cache = MarkdownCache::default();
    let content = "```rs\nfn main() {}\n```";

    let options = MarkdownRenderOptions {
        default_code_language: Some("sh"),
        code_language_overrides: &[],
    };

    let _ = run_markdown_frame_with_options(
        &ctx,
        headless_input(0.0),
        &mut cache,
        content,
        "explicit_tag",
        &options,
    );
    let output = run_markdown_frame_with_options(
        &ctx,
        headless_input(1.0),
        &mut cache,
        content,
        "explicit_tag",
        &options,
    );

    let (text_shapes, _, _) = count_shape_kinds(&output.shapes);
    assert!(
        text_shapes > 0,
        "explicit tag code block should render text"
    );
}

fn extract_font_sizes(shapes: &[egui::epaint::ClippedShape]) -> Vec<f32> {
    let mut sizes = Vec::new();
    for clipped in shapes {
        if let egui::epaint::Shape::Text(text_shape) = &clipped.shape {
            for section in &text_shape.galley.job.sections {
                let size = section.format.font_id.size;
                if size > 0.0 && !sizes.iter().any(|s: &f32| (s - size).abs() < 0.1) {
                    sizes.push(size);
                }
            }
        }
    }
    sizes
}

#[test]
fn inline_code_in_heading_is_larger_than_body_code() {
    let ctx = egui::Context::default();
    let mut cache = MarkdownCache::default();

    let heading_content = "# Heading `code`";
    let body_content = "Body `code`";
    let combined = format!("{heading_content}\n\n{body_content}");

    let _ = run_markdown_frame(
        &ctx,
        headless_input(0.0),
        &mut cache,
        &combined,
        "code_heading_size",
    );
    let output = run_markdown_frame(
        &ctx,
        headless_input(1.0),
        &mut cache,
        &combined,
        "code_heading_size",
    );

    let sizes = extract_font_sizes(&output.shapes);
    assert!(
        sizes.len() >= 2,
        "expected at least 2 distinct font sizes (heading vs body), got {sizes:?}"
    );

    let max_size = sizes.iter().copied().fold(f32::NEG_INFINITY, f32::max);
    let min_size = sizes.iter().copied().fold(f32::INFINITY, f32::min);
    assert!(
        max_size > min_size * 1.1,
        "heading code should be noticeably larger than body code, got sizes: {sizes:?}"
    );
}

fn extract_text_strings(shapes: &[egui::epaint::ClippedShape]) -> Vec<String> {
    shapes
        .iter()
        .filter_map(|clipped| match &clipped.shape {
            egui::epaint::Shape::Text(text_shape) => Some(text_shape.galley.job.text.clone()),
            _ => None,
        })
        .collect()
}

#[test]
fn ordered_list_double_digit_markers_render_all_numbers() {
    let ctx = egui::Context::default();
    let mut cache = MarkdownCache::default();

    let content = (1..=12)
        .map(|i| format!("{i}. Item {i}"))
        .collect::<Vec<_>>()
        .join("\n");

    let _ = run_markdown_frame(&ctx, headless_input(0.0), &mut cache, &content, "dbl_digit");
    let output = run_markdown_frame(&ctx, headless_input(1.0), &mut cache, &content, "dbl_digit");

    let texts = extract_text_strings(&output.shapes);
    for i in 10..=12 {
        let marker = format!("{i}.");
        assert!(
            texts.iter().any(|t| t.contains(&marker)),
            "missing marker '{marker}' in rendered text shapes: {texts:?}"
        );
    }
}

#[test]
fn ordered_list_marker_width_grows_with_digit_count() {
    let ctx = egui::Context::default();
    let mut cache = MarkdownCache::default();

    let short = "1. A\n2. B\n3. C";
    let long = (1..=20)
        .map(|i| format!("{i}. X"))
        .collect::<Vec<_>>()
        .join("\n");

    let _ = run_markdown_frame(&ctx, headless_input(0.0), &mut cache, short, "short_list");
    let short_out = run_markdown_frame(&ctx, headless_input(1.0), &mut cache, short, "short_list");

    let _ = run_markdown_frame(&ctx, headless_input(2.0), &mut cache, &long, "long_list");
    let long_out = run_markdown_frame(&ctx, headless_input(3.0), &mut cache, &long, "long_list");

    let short_texts = extract_text_strings(&short_out.shapes);
    let long_texts = extract_text_strings(&long_out.shapes);

    assert!(
        short_texts.iter().any(|t| t.contains("3.")),
        "short list should contain '3.' marker"
    );
    assert!(
        long_texts.iter().any(|t| t.contains("20.")),
        "long list should contain '20.' marker"
    );
}

#[test]
fn nested_unordered_list_renders_with_bullets() {
    let ctx = egui::Context::default();
    let mut cache = MarkdownCache::default();

    let content = "\
- Level 1
  - Level 2
    - Level 3";

    let _ = run_markdown_frame(
        &ctx,
        headless_input(0.0),
        &mut cache,
        content,
        "nested_bullets",
    );
    let output = run_markdown_frame(
        &ctx,
        headless_input(1.0),
        &mut cache,
        content,
        "nested_bullets",
    );

    let texts = extract_text_strings(&output.shapes);
    let joined = texts.join(" ");
    assert!(
        joined.contains("Level 1") && joined.contains("Level 2") && joined.contains("Level 3"),
        "nested list content missing, texts: {texts:?}"
    );
}

#[test]
fn list_continuation_paragraph_renders_in_egui() {
    let ctx = egui::Context::default();
    let mut cache = MarkdownCache::default();

    let content = "\
- First item with
  continuation text
- Second item";

    let _ = run_markdown_frame(
        &ctx,
        headless_input(0.0),
        &mut cache,
        content,
        "list_continuation",
    );
    let output = run_markdown_frame(
        &ctx,
        headless_input(1.0),
        &mut cache,
        content,
        "list_continuation",
    );

    let texts = extract_text_strings(&output.shapes);
    let joined = texts.join(" ");
    assert!(
        joined.contains("First item with")
            && joined.contains("continuation text")
            && joined.contains("Second item"),
        "list continuation missing, texts: {texts:?}"
    );
}

#[test]
fn ordered_list_paragraph_continuation_inside_same_item() {
    let ctx = egui::Context::default();
    let mut cache = MarkdownCache::default();

    let content = "\
1. First item

   Continuation paragraph

2. Second item";

    let _ = run_markdown_frame(
        &ctx,
        headless_input(0.0),
        &mut cache,
        content,
        "ordered_cont",
    );
    let output = run_markdown_frame(
        &ctx,
        headless_input(1.0),
        &mut cache,
        content,
        "ordered_cont",
    );

    let texts = extract_text_strings(&output.shapes);
    let joined = texts.join(" ");
    assert!(
        joined.contains("1.")
            && joined.contains("First item")
            && joined.contains("Continuation paragraph")
            && joined.contains("2.")
            && joined.contains("Second item"),
        "ordered list continuation missing, texts: {texts:?}"
    );
}

#[test]
fn mixed_ordered_and_unordered_nested_lists_render() {
    let ctx = egui::Context::default();
    let mut cache = MarkdownCache::default();

    let content = "\
1. Ordered item
   - Nested bullet
   - Another bullet
2. Second ordered";

    let _ = run_markdown_frame(
        &ctx,
        headless_input(0.0),
        &mut cache,
        content,
        "mixed_nested",
    );
    let output = run_markdown_frame(
        &ctx,
        headless_input(1.0),
        &mut cache,
        content,
        "mixed_nested",
    );

    let texts = extract_text_strings(&output.shapes);
    let joined = texts.join(" ");
    assert!(
        joined.contains("Ordered item")
            && joined.contains("Nested bullet")
            && joined.contains("Second ordered"),
        "mixed nested lists missing content, texts: {texts:?}"
    );
}

#[test]
fn table_renders_in_egui_with_alignment() {
    let ctx = egui::Context::default();
    let mut cache = MarkdownCache::default();

    let content = "| Left | Center | Right |\n| :--- | :---: | ---: |\n| A | B | C |";

    let _ = run_markdown_frame(
        &ctx,
        headless_input(0.0),
        &mut cache,
        content,
        "table_align",
    );
    let output = run_markdown_frame(
        &ctx,
        headless_input(1.0),
        &mut cache,
        content,
        "table_align",
    );

    let texts = extract_text_strings(&output.shapes);
    let joined = texts.join(" ");
    assert!(
        joined.contains("Left")
            && joined.contains("Center")
            && joined.contains("Right")
            && joined.contains("A")
            && joined.contains("B")
            && joined.contains("C"),
        "table content missing, texts: {texts:?}"
    );
}

#[test]
fn table_with_inline_formatting_renders_in_egui() {
    let ctx = egui::Context::default();
    let mut cache = MarkdownCache::default();

    let content = "| **Bold** | `code` |\n| --- | --- |\n| normal | test |";

    let _ = run_markdown_frame(
        &ctx,
        headless_input(0.0),
        &mut cache,
        content,
        "table_format",
    );
    let output = run_markdown_frame(
        &ctx,
        headless_input(1.0),
        &mut cache,
        content,
        "table_format",
    );

    let texts = extract_text_strings(&output.shapes);
    let joined = texts.join(" ");
    assert!(
        joined.contains("Bold")
            && joined.contains("code")
            && joined.contains("normal")
            && joined.contains("test"),
        "table with formatting missing content, texts: {texts:?}"
    );
}

#[test]
fn table_stays_within_layout_width_narrow() {
    let rect = render_markdown_painted_bounds(
        egui::vec2(200.0, 400.0),
        "| Header | Value |\n| --- | --- |\n| Long cell content | test |",
        "table_narrow",
    );

    assert!(
        rect.width() <= 200.0,
        "table width {} exceeds layout width 200",
        rect.width()
    );
}

#[test]
fn table_with_escaped_pipe_renders_in_egui() {
    let ctx = egui::Context::default();
    let mut cache = MarkdownCache::default();

    let content = "| Code | Value |\n| --- | --- |\n| `a | b` | test |";

    let _ = run_markdown_frame(
        &ctx,
        headless_input(0.0),
        &mut cache,
        content,
        "table_escape",
    );
    let output = run_markdown_frame(
        &ctx,
        headless_input(1.0),
        &mut cache,
        content,
        "table_escape",
    );

    let texts = extract_text_strings(&output.shapes);
    let joined = texts.join(" ");
    assert!(
        joined.contains("a | b") && joined.contains("test"),
        "escaped pipe in table missing, texts: {texts:?}"
    );
}

#[test]
fn definition_list_renders_in_egui() {
    let ctx = egui::Context::default();
    let mut cache = MarkdownCache::default();

    let content = "\
Term
: Definition text";

    let _ = run_markdown_frame(&ctx, headless_input(0.0), &mut cache, content, "def_list");
    let output = run_markdown_frame(&ctx, headless_input(1.0), &mut cache, content, "def_list");

    let texts = extract_text_strings(&output.shapes);
    let joined = texts.join(" ");
    assert!(
        joined.contains("Term") && joined.contains("Definition text"),
        "definition list content missing, texts: {texts:?}"
    );
}

#[test]
fn definition_list_with_multiple_definitions_in_egui() {
    let ctx = egui::Context::default();
    let mut cache = MarkdownCache::default();

    let content = "\
Term
: First
: Second";

    let _ = run_markdown_frame(
        &ctx,
        headless_input(0.0),
        &mut cache,
        content,
        "def_list_multi",
    );
    let output = run_markdown_frame(
        &ctx,
        headless_input(1.0),
        &mut cache,
        content,
        "def_list_multi",
    );

    let texts = extract_text_strings(&output.shapes);
    let joined = texts.join(" ");
    assert!(
        joined.contains("Term") && joined.contains("First") && joined.contains("Second"),
        "multiple definitions missing, texts: {texts:?}"
    );
}

#[test]
fn definition_list_with_inline_formatting_in_egui() {
    let ctx = egui::Context::default();
    let mut cache = MarkdownCache::default();

    let content = "\
**Bold Term**
: *Italic* definition";

    let _ = run_markdown_frame(
        &ctx,
        headless_input(0.0),
        &mut cache,
        content,
        "def_list_fmt",
    );
    let output = run_markdown_frame(
        &ctx,
        headless_input(1.0),
        &mut cache,
        content,
        "def_list_fmt",
    );

    let texts = extract_text_strings(&output.shapes);
    let joined = texts.join(" ");
    assert!(
        joined.contains("Bold") && joined.contains("Italic") && joined.contains("definition"),
        "formatted definition list missing, texts: {texts:?}"
    );
}

#[test]
#[ignore = "table cells currently render source text without resolved reference-style links"]
fn table_reference_link_in_egui() {
    // Reference link inside table cell with definition outside table
    let ctx = egui::Context::default();
    let mut cache = MarkdownCache::default();

    let content = "[link]: https://example.com

| A | B |
| --- | --- |
| [text][link] | normal |";

    let _ = run_markdown_frame(
        &ctx,
        headless_input(0.0),
        &mut cache,
        content,
        "table_ref_link",
    );
    let output = run_markdown_frame(
        &ctx,
        headless_input(1.0),
        &mut cache,
        content,
        "table_ref_link",
    );

    let texts = extract_text_strings(&output.shapes);
    let joined = texts.join(" ");
    // Should preserve reference definition and render link in table cell
    assert!(
        joined.contains("text") && joined.contains("example.com"),
        "table reference link missing, texts: {texts:?}"
    );
}

#[test]
fn ordered_list_continuation_with_nested_blocks_in_egui() {
    // Repeated 1. syntax with nested block should render correctly
    let ctx = egui::Context::default();
    let mut cache = MarkdownCache::default();

    let content = "\
1. First item

   Paragraph continuation

1. Second item";

    let _ = run_markdown_frame(
        &ctx,
        headless_input(0.0),
        &mut cache,
        content,
        "ordered_cont_nested",
    );
    let output = run_markdown_frame(
        &ctx,
        headless_input(1.0),
        &mut cache,
        content,
        "ordered_cont_nested",
    );

    let texts = extract_text_strings(&output.shapes);
    let joined = texts.join(" ");
    assert!(
        joined.contains("First item")
            && joined.contains("Paragraph continuation")
            && joined.contains("Second item"),
        "ordered list continuation missing, texts: {texts:?}"
    );
}
