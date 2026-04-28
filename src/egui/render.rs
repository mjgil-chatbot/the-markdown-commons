use super::inline::{
    render_inline_markdown, render_inline_nodes, InlineBaseStyle, InlineRenderStyle,
};
use super::MarkdownTable;
use crate::document::Block;
use crate::document::InlineContent;
use crate::document::ParagraphBlock;
use crate::inline::InlineNode;
use egui_extras::syntax_highlighting::{self, CodeTheme};
trait TableCellContent {
    fn source_text(&self) -> &str;

    fn inline_nodes(&self) -> Option<&[crate::inline::InlineNode]> {
        None
    }
}
impl TableCellContent for String {
    fn source_text(&self) -> &str {
        self
    }
}
impl TableCellContent for InlineContent {
    fn source_text(&self) -> &str {
        &self.source
    }

    fn inline_nodes(&self) -> Option<&[crate::inline::InlineNode]> {
        Some(&self.nodes)
    }
}
trait FromInlineContentCell: Sized {
    fn from_inline_content(content: &InlineContent) -> Self;
}
impl FromInlineContentCell for String {
    fn from_inline_content(content: &InlineContent) -> Self {
        content.source.clone()
    }
}
impl FromInlineContentCell for InlineContent {
    fn from_inline_content(content: &InlineContent) -> Self {
        content.clone()
    }
}
/// Options for markdown rendering.
#[derive(Default, Clone, Debug)]
pub struct MarkdownRenderOptions<'a> {
    /// Language to use for fenced code blocks that have no language tag.
    /// For example, `Some("sh")` treats bare ``` blocks as shell.
    pub default_code_language: Option<&'a str>,
    /// Map specific language tags to a different language for highlighting.
    /// For example, `&[("text", "sh"), ("txt", "sh")]` treats ````text`
    /// blocks as shell. Entries are checked before syntect receives the tag.
    pub code_language_overrides: &'a [(&'a str, &'a str)],
}
fn collect_ordered_list_continuation(
    blocks: &[Block],
    start_index: usize,
    current_indent: usize,
    expected_start: usize,
) -> (&[Block], Option<usize>, usize) {
    let mut index = start_index;

    while index < blocks.len() {
        match &blocks[index] {
            Block::OrderedList(list)
                if list.indent == current_indent && list.start == expected_start =>
            {
                return (&blocks[start_index..index], Some(index), index);
            }
            Block::OrderedList(list) if list.indent <= current_indent => break,
            Block::Paragraph(paragraph) if is_section_label_paragraph(&paragraph.content.nodes) => {
                break;
            }
            Block::Heading(_) | Block::HorizontalRule => break,
            _ => index += 1,
        }
    }

    (&blocks[start_index..index], None, index)
}
fn collect_unordered_list_continuation(
    blocks: &[Block],
    start_index: usize,
    current_indent: usize,
) -> (&[Block], Option<usize>, usize) {
    let mut index = start_index;

    while index < blocks.len() {
        match &blocks[index] {
            Block::UnorderedList(list) if list.indent == current_indent => {
                return (&blocks[start_index..index], Some(index), index);
            }
            Block::UnorderedList(list) if list.indent > current_indent => index += 1,
            Block::OrderedList(list) if list.indent > current_indent => index += 1,
            Block::Paragraph(_) | Block::Heading(_) | Block::HorizontalRule => break,
            Block::UnorderedList(_) | Block::OrderedList(_) => break,
            _ => break,
        }
    }

    (&blocks[start_index..index], None, index)
}
fn is_section_label_paragraph(nodes: &[InlineNode]) -> bool {
    matches!(nodes, [InlineNode::Strong(children)] if matches!(children.as_slice(), [InlineNode::Text(_)]))
}
fn render_paragraph_block(ui: &mut egui::Ui, paragraph: &ParagraphBlock, quote: bool) {
    render_inline_nodes(
        ui,
        &paragraph.content.nodes,
        InlineRenderStyle {
            base: InlineBaseStyle::Normal,
            heading: None,
            quote,
        },
    );
}
fn render_heading(
    ui: &mut egui::Ui,
    level: usize,
    nodes: &[crate::inline::InlineNode],
    quote: bool,
) {
    let heading_level = (level.saturating_sub(1)).min(5) as u8;
    render_inline_nodes(
        ui,
        nodes,
        InlineRenderStyle {
            base: InlineBaseStyle::Normal,
            heading: Some(heading_level),
            quote,
        },
    );
}
#[allow(dead_code)]
fn render_paragraph(ui: &mut egui::Ui, text: &str, quote: bool) {
    render_inline_markdown(
        ui,
        text,
        InlineRenderStyle {
            base: InlineBaseStyle::Normal,
            heading: None,
            quote,
        },
        &Default::default(),
    );
}
fn render_code_block(
    ui: &mut egui::Ui,
    id_salt: &str,
    language: Option<&str>,
    code: &str,
    options: &MarkdownRenderOptions<'_>,
) {
    let outer_width = ui.available_width();
    let inner_width = (outer_width - 16.0).max(0.0);
    let theme = CodeTheme::from_style(ui.style());
    let mono_font = egui::TextStyle::Monospace.resolve(ui.style());
    let code_line_height = mono_font.size * 1.6;
    // Apply caller-provided overrides, then fall back to the default.
    let effective_language = match language {
        Some(tag) => options
            .code_language_overrides
            .iter()
            .find(|(from, _)| *from == tag)
            .map(|(_, to)| *to)
            .unwrap_or(tag),
        None => options.default_code_language.unwrap_or(""),
    };
    let wrap_json = effective_language == "json";
    let trimmed_code = code.strip_suffix('\n').unwrap_or(code);
    let mut job = syntax_highlighting::highlight(
        ui.ctx(),
        ui.style(),
        &theme,
        trimmed_code,
        effective_language,
    );
    job.wrap.max_width = if wrap_json {
        inner_width
    } else {
        f32::INFINITY
    };
    for section in &mut job.sections {
        section.format.line_height = Some(code_line_height);
    }

    ui.allocate_ui(egui::vec2(outer_width, 0.0), |ui| {
        ui.set_width(outer_width);
        ui.set_min_width(outer_width);
        ui.set_max_width(outer_width);

        egui::Frame::none()
            .fill(ui.visuals().extreme_bg_color)
            .stroke(ui.visuals().widgets.noninteractive.bg_stroke)
            .rounding(ui.style().noninteractive().rounding)
            .inner_margin(egui::Margin::same(8.0))
            .show(ui, |ui| {
                let frame_width = ui.available_width();
                let frame_rect = ui.max_rect();
                ui.set_min_width(frame_width);
                ui.set_max_width(frame_width);

                if wrap_json {
                    ui.add(
                        egui::Label::new(job.clone())
                            .selectable(true)
                            .wrap_mode(egui::TextWrapMode::Wrap),
                    );
                } else {
                    egui::ScrollArea::horizontal()
                        .max_width(frame_width)
                        .id_salt(format!("{}_scroll", id_salt))
                        .show(ui, |ui| {
                            ui.add(
                                egui::Label::new(job.clone())
                                    .selectable(true)
                                    .wrap_mode(egui::TextWrapMode::Extend),
                            );
                        });
                }

                let spacing = &ui.style().spacing;
                let button_position = egui::pos2(
                    frame_rect.right_top().x - spacing.icon_width * 0.5 - spacing.button_padding.x,
                    frame_rect.right_top().y + spacing.button_padding.y * 2.0,
                );
                let persistent_id = ui.make_persistent_id(format!("{}_copy_button", id_salt));
                let copied_icon = ui.memory_mut(|memory| {
                    *memory.data.get_temp_mut_or_default::<bool>(persistent_id)
                });
                let copy_button = ui
                    .put(
                        egui::Rect {
                            min: button_position,
                            max: button_position,
                        },
                        egui::Button::new(if copied_icon { "✔" } else { "🗐" })
                            .small()
                            .frame(false)
                            .fill(egui::Color32::TRANSPARENT),
                    )
                    .on_hover_cursor(
                        ui.visuals()
                            .interact_cursor
                            .unwrap_or(egui::CursorIcon::Default),
                    );

                if copied_icon && !copy_button.hovered() {
                    ui.memory_mut(|memory| {
                        *memory.data.get_temp_mut_or_default(persistent_id) = false;
                    });
                }
                if !copied_icon && copy_button.clicked() {
                    ui.memory_mut(|memory| {
                        *memory.data.get_temp_mut_or_default(persistent_id) = true;
                    });
                }
                if copy_button.clicked() {
                    ui.ctx().copy_text(trimmed_code.to_owned());
                }
            });
    });
}
fn render_table(ui: &mut egui::Ui, table: &MarkdownTable, id_salt: &str) {
    let num_cols = table.headers.len();
    if num_cols == 0 {
        return;
    }

    let outer_width = ui.available_width();
    ui.allocate_ui(egui::vec2(outer_width, 0.0), |ui| {
        ui.set_width(outer_width);
        ui.set_min_width(outer_width);
        ui.set_max_width(outer_width);

        egui::Frame::group(ui.style()).show(ui, |ui| {
            let frame_width = ui.available_width();
            ui.set_min_width(frame_width);
            ui.set_max_width(frame_width);

            let col_widths = calculate_column_widths(table, ui, frame_width);
            let total_width: f32 = col_widths.iter().sum::<f32>() + (num_cols as f32 * 8.0);

            if total_width > frame_width {
                egui::ScrollArea::horizontal()
                    .max_width(frame_width)
                    .id_salt(format!("{}_scroll", id_salt))
                    .show(ui, |ui| {
                        render_table_grid(ui, table, id_salt, &col_widths);
                    });
            } else {
                render_table_grid(ui, table, id_salt, &col_widths);
            }
        });
    });
}
fn render_table_grid(ui: &mut egui::Ui, table: &MarkdownTable, id_salt: &str, col_widths: &[f32]) {
    let num_cols = table.headers.len();

    egui::Grid::new(id_salt)
        .num_columns(num_cols)
        .spacing([8.0, 4.0])
        .striped(true)
        .min_col_width(80.0)
        .show(ui, |ui| {
            for (i, header) in table.headers.iter().enumerate() {
                let cell_width = col_widths.get(i).copied().unwrap_or(80.0);
                with_table_cell_ui(ui, cell_width, |ui| {
                    render_cell_markdown(ui, header, true);
                });
            }
            ui.end_row();

            for _ in 0..num_cols {
                ui.separator();
            }
            ui.end_row();

            for row in &table.rows {
                for (col_idx, cell) in row.iter().enumerate() {
                    let cell_width = col_widths.get(col_idx).copied().unwrap_or(80.0);
                    with_table_cell_ui(ui, cell_width, |ui| {
                        render_cell_markdown(ui, cell, false);
                    });
                }
                ui.end_row();
            }
        });
}
fn with_table_cell_ui(ui: &mut egui::Ui, width: f32, render: impl FnOnce(&mut egui::Ui)) {
    ui.scope(|ui| {
        ui.set_width(width);
        ui.set_min_width(width);
        ui.set_max_width(width);
        ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Wrap);
        render(ui);
    });
}
fn render_cell_markdown<T: TableCellContent>(ui: &mut egui::Ui, content: &T, is_header: bool) {
    let render_style = InlineRenderStyle {
        base: if is_header {
            InlineBaseStyle::Strong
        } else {
            InlineBaseStyle::Normal
        },
        heading: None,
        quote: false,
    };

    if let Some(nodes) = content.inline_nodes() {
        render_inline_nodes(ui, nodes, render_style);
    } else {
        render_inline_markdown(ui, content.source_text(), render_style, &Default::default());
    }
}
pub(crate) fn count_emoji(s: &str) -> usize {
    s.chars()
        .filter(|c| {
            let c = *c as u32;
            (0x1F300..=0x1F9FF).contains(&c)
                || (0x2600..=0x26FF).contains(&c)
                || (0x2700..=0x27BF).contains(&c)
                || (0x2300..=0x23FF).contains(&c)
                || c == 0x2705
                || c == 0x274C
        })
        .count()
}
fn calculate_text_width_with_code(text: &str, ui: &egui::Ui) -> f32 {
    let body_font = egui::TextStyle::Body.resolve(ui.style());
    let mono_font = egui::TextStyle::Monospace.resolve(ui.style());
    let mut total_width: f32 = 0.0;
    let mut remaining = text;

    while !remaining.is_empty() {
        if let Some(start) = remaining.find('`') {
            if start > 0 {
                let before = &remaining[..start];
                let galley = ui.fonts(|fonts| {
                    fonts.layout_no_wrap(
                        before.to_string(),
                        body_font.clone(),
                        egui::Color32::WHITE,
                    )
                });
                total_width += galley.rect.width();
            }

            let after_start = &remaining[start + 1..];
            if let Some(end) = after_start.find('`') {
                let code = &after_start[..end];
                let galley = ui.fonts(|fonts| {
                    fonts.layout_no_wrap(code.to_string(), mono_font.clone(), egui::Color32::WHITE)
                });
                total_width += galley.rect.width() + 10.0;
                remaining = &after_start[end + 1..];
            } else {
                let galley = ui.fonts(|fonts| {
                    fonts.layout_no_wrap(
                        remaining[start..].to_string(),
                        body_font.clone(),
                        egui::Color32::WHITE,
                    )
                });
                total_width += galley.rect.width();
                break;
            }
        } else {
            let galley = ui.fonts(|fonts| {
                fonts.layout_no_wrap(
                    remaining.to_string(),
                    body_font.clone(),
                    egui::Color32::WHITE,
                )
            });
            total_width += galley.rect.width();
            break;
        }
    }

    total_width
}
fn calculate_column_widths(table: &MarkdownTable, ui: &egui::Ui, available_width: f32) -> Vec<f32> {
    let font_id = egui::TextStyle::Body.resolve(ui.style());
    let mut widths: Vec<f32> = vec![0.0; table.headers.len()];

    for (i, header) in table.headers.iter().enumerate() {
        let header_text = header.source_text();
        let galley = ui.fonts(|fonts| {
            fonts.layout_no_wrap(
                header_text.to_string(),
                font_id.clone(),
                egui::Color32::WHITE,
            )
        });
        let emoji_count = count_emoji(header_text);
        let emoji_extra = emoji_count as f32 * 10.0;
        widths[i] = widths[i].max(galley.rect.width() + 24.0 + emoji_extra);
    }

    for row in &table.rows {
        for (i, cell) in row.iter().enumerate() {
            if i < widths.len() {
                let cell_text = cell.source_text();
                let text_width = if cell_text.contains('`') {
                    calculate_text_width_with_code(cell_text, ui)
                } else {
                    let galley = ui.fonts(|fonts| {
                        fonts.layout_no_wrap(
                            cell_text.to_string(),
                            font_id.clone(),
                            egui::Color32::WHITE,
                        )
                    });
                    galley.rect.width()
                };

                let emoji_count = count_emoji(cell_text);
                let emoji_extra = emoji_count as f32 * 10.0;
                widths[i] = widths[i].max(text_width + 24.0 + emoji_extra);
            }
        }
    }

    for width in &mut widths {
        *width = width.min(400.0).max(80.0);
    }

    let num_cols = widths.len().max(1);
    let spacing_total = num_cols as f32 * 8.0;
    let available_for_columns = (available_width - spacing_total).max(0.0);
    let total_width: f32 = widths.iter().sum();
    if total_width > 0.0 && total_width < available_for_columns {
        let scale = available_for_columns / total_width;
        for width in &mut widths {
            *width *= scale;
        }
    }

    widths
}
