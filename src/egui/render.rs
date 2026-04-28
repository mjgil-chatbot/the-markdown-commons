use super::inline::{
    render_inline_markdown, render_inline_nodes, InlineBaseStyle, InlineRenderStyle,
};
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
