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
