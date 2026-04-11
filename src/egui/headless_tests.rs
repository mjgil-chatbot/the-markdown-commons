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
fn extract_text_strings(shapes: &[egui::epaint::ClippedShape]) -> Vec<String> {
    shapes
        .iter()
        .filter_map(|clipped| match &clipped.shape {
            egui::epaint::Shape::Text(text_shape) => Some(text_shape.galley.job.text.clone()),
            _ => None,
        })
        .collect()
}
