pub(crate) fn heading_font_size(ui: &egui::Ui, level: u8) -> f32 {
    let max_height = ui
        .style()
        .text_styles
        .get(&egui::TextStyle::Heading)
        .map_or(32.0, |font| font.size);
    let min_height = ui
        .style()
        .text_styles
        .get(&egui::TextStyle::Body)
        .map_or(14.0, |font| font.size);
    let diff = max_height - min_height;

    match level {
        0 => max_height,
        1 => min_height + diff * 0.835,
        2 => min_height + diff * 0.668,
        3 => min_height + diff * 0.501,
        4 => min_height + diff * 0.334,
        _ => min_height + diff * 0.167,
    }
}
pub(crate) fn height_body(ui: &egui::Ui) -> f32 {
    ui.text_style_height(&egui::TextStyle::Body)
}
