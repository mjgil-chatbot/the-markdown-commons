use super::inline::{render_inline_nodes, InlineBaseStyle, InlineRenderStyle};
use super::style::{height_body, width_body_space};
use crate::document::{DefinitionItem, ListItem as DocumentListItem, TaskItem};

pub(crate) fn render_document_list(
    ui: &mut egui::Ui,
    items: &[DocumentListItem],
    ordered_start: Option<usize>,
) {
    let marker_width = marker_width_for_list(ui, ordered_start, items.len());
    ui.scope(|ui| {
        ui.spacing_mut().item_spacing.y = 0.0;
        for (index, item) in items.iter().enumerate() {
            ui.horizontal_wrapped(|ui| {
                render_document_list_marker(ui, ordered_start, index, marker_width);
                ui.add_space(4.0);
                render_inline_nodes(ui, &item.content.nodes, InlineRenderStyle::normal());
            });
        }
    });
}

pub(crate) fn render_task_list(ui: &mut egui::Ui, items: &[TaskItem]) {
    ui.scope(|ui| {
        ui.spacing_mut().item_spacing.y = 0.0;
        for item in items {
            ui.horizontal_wrapped(|ui| {
                let marker = if item.checked { "☑" } else { "☐" };
                ui.label(marker);
                ui.add_space(4.0);
                render_inline_nodes(ui, &item.content.nodes, InlineRenderStyle::normal());
            });
        }
    });
}

pub(crate) fn render_definition_list(ui: &mut egui::Ui, items: &[DefinitionItem], quote: bool) {
    ui.scope(|ui| {
        ui.spacing_mut().item_spacing.y = 2.0;
        for item in items {
            render_inline_nodes(
                ui,
                &item.term.nodes,
                InlineRenderStyle {
                    base: InlineBaseStyle::Strong,
                    heading: None,
                    quote,
                },
            );
            for definition in &item.definitions {
                ui.horizontal_wrapped(|ui| {
                    ui.add_space(width_body_space(ui) * 2.0);
                    render_inline_nodes(ui, &definition.nodes, InlineRenderStyle::normal());
                });
            }
        }
    });
}

fn marker_width_for_list(ui: &egui::Ui, ordered_start: Option<usize>, count: usize) -> f32 {
    if let Some(start) = ordered_start {
        let max_number = start + count.saturating_sub(1);
        let widest_marker = format!("{}.", max_number);
        let font_id = egui::TextStyle::Body.resolve(ui.style());
        let text_width: f32 = widest_marker
            .chars()
            .map(|ch| ui.fonts(|fonts| fonts.glyph_width(&font_id, ch)))
            .sum();
        text_width + width_body_space(ui)
    } else {
        width_body_space(ui) * 4.0
    }
}

fn render_document_list_marker(
    ui: &mut egui::Ui,
    ordered_start: Option<usize>,
    index: usize,
    marker_width: f32,
) {
    let marker_height = height_body(ui);
    let (rect, _) = ui.allocate_exact_size(
        egui::vec2(marker_width, marker_height),
        egui::Sense::hover(),
    );

    if let Some(start) = ordered_start {
        let marker = format!("{}.", start + index);
        ui.painter().text(
            rect.right_center(),
            egui::Align2::RIGHT_CENTER,
            marker,
            egui::TextStyle::Body.resolve(ui.style()),
            ui.visuals().strong_text_color(),
        );
    } else {
        ui.painter().circle_filled(
            rect.center(),
            rect.height() / 6.0,
            ui.visuals().strong_text_color(),
        );
    }
}
