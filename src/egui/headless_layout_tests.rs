use super::inline::{layout_job_from_inline_markdown, InlineRenderStyle};
use super::*;

fn headless_input(time: f64) -> egui::RawInput {
    egui::RawInput {
        screen_rect: Some(egui::Rect::from_min_size(
            egui::Pos2::ZERO,
            egui::vec2(800.0, 600.0),
        )),
        time: Some(time),
        ..Default::default()
    }
}

fn render_markdown_bounds(screen_size: egui::Vec2, content: &str, id_salt: &str) -> egui::Rect {
    let ctx = egui::Context::default();
    let mut cache = MarkdownCache::default();

    let output = ctx.run(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, screen_size)),
            time: Some(0.0),
            ..Default::default()
        },
        |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                render_markdown(ui, &mut cache, content, id_salt);
            });
        },
    );
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
}

fn render_markdown_summary(screen_size: egui::Vec2, content: &str, id_salt: &str) -> RenderSummary {
    let ctx = egui::Context::default();
    let mut cache = MarkdownCache::default();
    let output = ctx.run(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, screen_size)),
            time: Some(0.0),
            ..Default::default()
        },
        |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                render_markdown(ui, &mut cache, content, id_salt);
            });
        },
    );
    let (text_shapes, line_segments, rect_shapes) = count_shape_kinds(&output.shapes);

    RenderSummary {
        text_shapes,
        line_segments,
        rect_shapes,
    }
}

#[test]
fn test_headless_table_stays_within_narrow_layout_width() {
    let rect = render_markdown_bounds(
        egui::vec2(240.0, 600.0),
        "| wide header | value |\n| --- | --- |\n| a very long table cell that should not overflow the layout | 2 |",
        "headless_table_width",
    );

    assert!(rect.width() <= 240.0);
}

#[test]
fn test_headless_complex_document_renders_stable_multi_block_summary() {
    let content = r#"# Release Notes

Paragraph with **bold**, `inline code`, and a [link](https://example.com).

> Quoted text
>
> Follow-up line.

- Alpha
- Beta

| Item | Status |
| --- | --- |
| Parser | done |
| Renderer | active |

```json
{"enabled":true,"items":["alpha","beta","gamma"]}
```"#;
    let summary = render_markdown_summary(egui::vec2(420.0, 900.0), content, "complex_document");
    let bounds = render_markdown_bounds(egui::vec2(420.0, 900.0), content, "complex_document");

    assert!(summary.text_shapes >= 6);
    assert!(summary.line_segments >= 1);
    assert!(summary.rect_shapes >= 1);
    assert!(bounds.width() <= 420.0);
}

#[test]
fn test_headless_narrow_json_code_block_stays_within_layout_width() {
    let rect = render_markdown_bounds(
        egui::vec2(220.0, 500.0),
        "```json\n{\"very_long_key\":\"this should wrap instead of overflowing the parent width\"}\n```",
        "json_wrap_width",
    );

    assert!(rect.width() <= 220.0);
}

#[test]
fn test_headless_narrow_mixed_document_stays_within_layout_width() {
    let rect = render_markdown_bounds(
        egui::vec2(260.0, 800.0),
        "## Heading\n\nA narrow paragraph with enough words to wrap across multiple rows.\n\n| Name | Value |\n| --- | --- |\n| very long cell content that must stay clipped | 2 |\n\n> blockquote",
        "mixed_narrow_width",
    );

    assert!(rect.width() <= 260.0);
}

#[test]
fn mixed_inline_markdown_renders_as_single_text_shape() {
    let summary = render_markdown_summary(
        egui::vec2(260.0, 400.0),
        "Paragraph with **bold**, `code`, and a [link](https://example.com) that wraps onto another line for selection testing.",
        "single_inline_shape",
    );

    assert_eq!(summary.text_shapes, 1);
}

#[test]
fn test_inline_layout_wraps_into_more_rows_when_narrow() {
    let ctx = egui::Context::default();
    let content = "This is a long paragraph with enough words to force wrapping in a narrow layout but stay on fewer lines in a wider layout.";
    let mut row_counts = (0, 0);

    let _ = ctx.run(headless_input(0.0), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            let wide_job =
                layout_job_from_inline_markdown(ui, content, 800.0, InlineRenderStyle::normal());
            let narrow_job =
                layout_job_from_inline_markdown(ui, content, 120.0, InlineRenderStyle::normal());

            let wide = ui.fonts(|fonts| fonts.layout_job(wide_job));
            let narrow = ui.fonts(|fonts| fonts.layout_job(narrow_job));
            row_counts = (wide.rows.len(), narrow.rows.len());
        });
    });

    assert!(row_counts.1 > row_counts.0);
}
