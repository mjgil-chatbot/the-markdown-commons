#![cfg(feature = "egui")]

use tmc::{render_markdown, MarkdownCache};

fn headless_input(screen_size: egui::Vec2) -> egui::RawInput {
    egui::RawInput {
        screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, screen_size)),
        time: Some(0.0),
        ..Default::default()
    }
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

fn painted_width(
    ctx: &egui::Context,
    shapes: Vec<egui::epaint::ClippedShape>,
    pixels_per_point: f32,
) -> f32 {
    let primitives = ctx.tessellate(shapes, pixels_per_point);
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

    bounds.width()
}

fn render_snapshot_summary(screen_size: egui::Vec2, content: &str, id_salt: &str) -> String {
    let ctx = egui::Context::default();
    let mut cache = MarkdownCache::default();
    let output = ctx.run(headless_input(screen_size), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            render_markdown(ui, &mut cache, content, id_salt);
        });
    });

    let (text_shapes, line_segments, rect_shapes) = count_shape_kinds(&output.shapes);
    let clipped_shapes = output.shapes.len();
    let painted_width = painted_width(&ctx, output.shapes, output.pixels_per_point);

    format!(
        "screen_width={screen_width}\nclipped_shapes={clipped_shapes}\ntext_shapes={text_shapes}\nline_segments={line_segments}\nrect_shapes={rect_shapes}\npainted_width={painted_width:.2}\n",
        screen_width = screen_size.x,
    )
}

#[test]
fn mixed_document_snapshot() {
    let summary = render_snapshot_summary(
        egui::vec2(420.0, 900.0),
        r#"# Release Notes

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
```"#,
        "snapshot_mixed_document",
    );

    insta::assert_snapshot!("egui_mixed_document", summary);
}

#[test]
fn narrow_layout_snapshot() {
    let summary = render_snapshot_summary(
        egui::vec2(260.0, 800.0),
        "## Heading\n\nA narrow paragraph with enough words to wrap across multiple rows.\n\n| Name | Value |\n| --- | --- |\n| very long cell content that must stay clipped | 2 |\n\n> blockquote",
        "snapshot_narrow_layout",
    );

    insta::assert_snapshot!("egui_narrow_layout", summary);
}
