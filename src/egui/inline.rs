use std::ops::Range;

use super::style::{heading_font_size, height_body};
use crate::inline::{parse_inline_nodes, InlineNode, LinkDefinitions};
use pulldown_cmark::{Event, Options, Parser, Tag, TagEnd};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct InlineSpan {
    pub(crate) text: String,
    pub(crate) is_code: bool,
    pub(crate) is_bold: bool,
}

pub(crate) fn parse_inline_formatting(content: &str) -> Vec<InlineSpan> {
    let mut spans = Vec::new();
    let mut buffer = String::new();
    let mut in_code = false;
    let mut in_bold = false;
    let mut idx = 0;

    while idx < content.len() {
        let remaining = &content[idx..];
        if remaining.starts_with('`') {
            if !buffer.is_empty() {
                spans.push(InlineSpan {
                    text: std::mem::take(&mut buffer),
                    is_code: in_code,
                    is_bold: in_bold,
                });
            }
            in_code = !in_code;
            idx += 1;
            continue;
        }

        if !in_code && remaining.starts_with("**") {
            if !buffer.is_empty() {
                spans.push(InlineSpan {
                    text: std::mem::take(&mut buffer),
                    is_code: in_code,
                    is_bold: in_bold,
                });
            }
            in_bold = !in_bold;
            idx += 2;
            continue;
        }

        let ch = remaining.chars().next().unwrap();
        buffer.push(ch);
        idx += ch.len_utf8();
    }

    if !buffer.is_empty() {
        spans.push(InlineSpan {
            text: buffer,
            is_code: in_code,
            is_bold: in_bold,
        });
    }

    spans
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) enum InlineBaseStyle {
    Normal,
    Strong,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) struct InlineRenderStyle {
    pub(crate) base: InlineBaseStyle,
    pub(crate) heading: Option<u8>,
    pub(crate) quote: bool,
}

impl InlineRenderStyle {
    pub(crate) fn normal() -> Self {
        Self {
            base: InlineBaseStyle::Normal,
            heading: None,
            quote: false,
        }
    }
}

#[derive(Default, Clone)]
struct WidgetStyle {
    heading: Option<u8>,
    strong: bool,
    emphasis: bool,
    strikethrough: bool,
    quote: bool,
    code: bool,
}

impl WidgetStyle {
    fn to_richtext(&self, ui: &egui::Ui, text: &str) -> egui::RichText {
        let mut text = egui::RichText::new(text);
        let body_font = egui::TextStyle::Body.resolve(ui.style());
        let mut line_height = height_body(ui);

        if let Some(level) = self.heading {
            match level {
                0 => {
                    text = text.strong().heading();
                    line_height = heading_font_size(ui, level) * 1.15;
                }
                1..=5 => {
                    text = text.size(heading_font_size(ui, level));
                    line_height = heading_font_size(ui, level) * 1.15;
                    if level <= 3 {
                        text = text.strong();
                    }
                }
                _ => {}
            }
        }

        if self.quote {
            text = text.weak();
        }
        if self.strong {
            text = text.strong();
        }
        if self.emphasis {
            text = text.italics();
        }
        if self.strikethrough {
            text = text.strikethrough();
        }
        if self.code {
            let code_size = match self.heading {
                Some(level @ 0..=5) => heading_font_size(ui, level) * 0.88,
                _ => body_font.size * 0.88,
            };
            text = text
                .code()
                .size(code_size)
                .background_color(egui::Color32::from_rgb(241, 245, 249))
                .color(egui::Color32::from_rgb(13, 148, 136));
        }

        text.line_height(Some(line_height))
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct InlineLinkSpan {
    destination: String,
    char_range: Range<usize>,
    byte_range: Range<usize>,
    run_id: usize,
}

struct InlineLayout {
    job: egui::text::LayoutJob,
    links: Vec<InlineLinkSpan>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct HoveredLinkRowRange {
    row_index: usize,
    global_char_range: Range<usize>,
    column_range: Range<usize>,
}

fn open_link_destination(ui: &egui::Ui, destination: &str) {
    ui.ctx()
        .open_url(egui::OpenUrl::new_tab(destination.to_string()));
}

fn build_inline_layout(ui: &egui::Ui, fragments: &[FlatInlineFragment]) -> InlineLayout {
    let style = ui.style().clone();
    let mut job = egui::text::LayoutJob::default();

    for fragment in fragments {
        if fragment.text.is_empty() {
            continue;
        }

        let mut rich_text = fragment.style.to_richtext(ui, &fragment.text);
        if fragment.link.is_some() {
            rich_text = rich_text.color(ui.visuals().hyperlink_color);
        }
        rich_text.append_to(
            &mut job,
            &style,
            egui::FontSelection::Default,
            ui.text_valign(),
        );
    }

    job.wrap = egui::text::TextWrapping::wrap_at_width(ui.available_width().max(0.0));

    InlineLayout {
        job,
        links: collect_link_spans(fragments),
    }
}

fn collect_link_spans(fragments: &[FlatInlineFragment]) -> Vec<InlineLinkSpan> {
    let mut spans: Vec<InlineLinkSpan> = Vec::new();
    let mut char_index = 0usize;
    let mut byte_index = 0usize;

    for fragment in fragments {
        let next_char_index = char_index + fragment.text.chars().count();
        let next_byte_index = byte_index + fragment.text.len();

        if let (Some(destination), Some(run_id)) = (&fragment.link, fragment.link_run_id) {
            if char_index != next_char_index {
                if let Some(last) = spans.last_mut() {
                    if last.run_id == run_id {
                        last.char_range.end = next_char_index;
                        last.byte_range.end = next_byte_index;
                    } else {
                        spans.push(InlineLinkSpan {
                            destination: destination.clone(),
                            char_range: char_index..next_char_index,
                            byte_range: byte_index..next_byte_index,
                            run_id,
                        });
                    }
                } else {
                    spans.push(InlineLinkSpan {
                        destination: destination.clone(),
                        char_range: char_index..next_char_index,
                        byte_range: byte_index..next_byte_index,
                        run_id,
                    });
                }
            }
        }

        char_index = next_char_index;
        byte_index = next_byte_index;
    }

    spans
}

fn render_selectable_inline(ui: &mut egui::Ui, inline: InlineLayout) -> egui::Response {
    let label = egui::Label::new(inline.job.clone())
        .selectable(true)
        .wrap_mode(egui::TextWrapMode::Wrap)
        .sense(if inline.links.is_empty() {
            egui::Sense::hover()
        } else {
            egui::Sense::click()
        });
    let (galley_pos, galley, response) = label.layout_in_ui(ui);
    let hovered_link = hovered_inline_link(&response, galley_pos, &galley, &inline.links);
    let response = if hovered_link.is_some() {
        response.on_hover_cursor(egui::CursorIcon::PointingHand)
    } else {
        response
    };

    if ui.is_rect_visible(response.rect) {
        egui::text_selection::LabelSelectionState::label_text_selection(
            ui,
            &response,
            galley_pos,
            galley.clone(),
            ui.visuals().text_color(),
            egui::Stroke::NONE,
        );

        if let Some(link) = hovered_link.as_ref() {
            paint_hovered_link_underline(ui, galley_pos, &galley, &inline.job, link);
        }
    }

    if let Some(link) = hovered_link {
        if response.clicked() || response.middle_clicked() {
            open_link_destination(ui, &link.destination);
        }
    }

    response
}

fn ranges_overlap(left: &Range<usize>, right: &Range<usize>) -> bool {
    left.start < right.end && right.start < left.end
}

fn hovered_link_row_ranges(
    galley: &egui::Galley,
    link: &InlineLinkSpan,
) -> Vec<HoveredLinkRowRange> {
    let mut row_ranges = Vec::new();
    let mut row_start = 0usize;

    for (row_index, row) in galley.rows.iter().enumerate() {
        let row_end = row_start + row.char_count_excluding_newline();
        let hovered_start = row_start.max(link.char_range.start);
        let hovered_end = row_end.min(link.char_range.end);

        if hovered_start < hovered_end {
            row_ranges.push(HoveredLinkRowRange {
                row_index,
                global_char_range: hovered_start..hovered_end,
                column_range: (hovered_start - row_start)..(hovered_end - row_start),
            });
        }

        row_start = row_end + usize::from(row.ends_with_newline);
    }

    row_ranges
}

fn hovered_link_stroke(layout_job: &egui::text::LayoutJob, link: &InlineLinkSpan) -> egui::Stroke {
    layout_job
        .sections
        .iter()
        .find(|section| ranges_overlap(&section.byte_range, &link.byte_range))
        .map(|section| egui::Stroke::new(1.0, section.format.color))
        .unwrap_or(egui::Stroke::NONE)
}

fn paint_hovered_link_underline(
    ui: &egui::Ui,
    galley_pos: egui::Pos2,
    galley: &egui::Galley,
    layout_job: &egui::text::LayoutJob,
    link: &InlineLinkSpan,
) {
    let stroke = hovered_link_stroke(layout_job, link);
    if stroke == egui::Stroke::NONE {
        return;
    }

    for row_range in hovered_link_row_ranges(galley, link) {
        let row = &galley.rows[row_range.row_index];
        let start_x = row.x_offset(row_range.column_range.start);
        let end_x = row.x_offset(row_range.column_range.end);
        // Keep the underline inside the row so hover feedback cannot expand
        // the rendered bounds or disturb surrounding layout.
        let underline_y = row.max_y() - stroke.width.max(1.0);

        ui.painter().line_segment(
            [
                galley_pos + egui::vec2(start_x, underline_y),
                galley_pos + egui::vec2(end_x, underline_y),
            ],
            stroke,
        );
    }
}

fn hovered_inline_link<'a>(
    response: &egui::Response,
    galley_pos: egui::Pos2,
    galley: &egui::Galley,
    links: &'a [InlineLinkSpan],
) -> Option<&'a InlineLinkSpan> {
    let pointer_pos = response
        .interact_pointer_pos()
        .or_else(|| response.hover_pos())?;
    let cursor = galley.cursor_from_pos(pointer_pos - galley_pos).ccursor;
    let char_index = cursor.index;

    links
        .iter()
        .find(|link| link.char_range.contains(&char_index))
        .or_else(|| {
            char_index
                .checked_sub(1)
                .and_then(|index| links.iter().find(|link| link.char_range.contains(&index)))
        })
}

pub(crate) fn render_inline_markdown(
    ui: &mut egui::Ui,
    content: &str,
    render_style: InlineRenderStyle,
    link_defs: &LinkDefinitions,
) {
    let nodes = parse_inline_nodes(content, link_defs);
    render_inline_nodes(ui, &nodes, render_style);
}

pub(crate) fn render_inline_nodes(
    ui: &mut egui::Ui,
    nodes: &[InlineNode],
    render_style: InlineRenderStyle,
) {
    let mut fragments = Vec::new();
    let mut next_link_run_id = 0usize;
    let base_style = WidgetStyle {
        heading: render_style.heading,
        strong: render_style.base == InlineBaseStyle::Strong,
        quote: render_style.quote,
        ..Default::default()
    };
    flatten_inline_nodes(
        &mut fragments,
        nodes,
        &base_style,
        None,
        None,
        &mut next_link_run_id,
    );
    render_selectable_inline(ui, build_inline_layout(ui, &fragments));
}

#[derive(Clone)]
struct FlatInlineFragment {
    text: String,
    style: WidgetStyle,
    link: Option<String>,
    link_run_id: Option<usize>,
}

fn flatten_inline_nodes(
    out: &mut Vec<FlatInlineFragment>,
    nodes: &[InlineNode],
    style: &WidgetStyle,
    link: Option<&str>,
    link_run_id: Option<usize>,
    next_link_run_id: &mut usize,
) {
    for node in nodes {
        match node {
            InlineNode::Text(text) | InlineNode::Entity(text) | InlineNode::RawHtml(text) => {
                out.push(FlatInlineFragment {
                    text: text.clone(),
                    style: style.clone(),
                    link: link.map(str::to_string),
                    link_run_id,
                });
            }
            InlineNode::Code(text) => {
                let mut style = style.clone();
                style.code = true;
                out.push(FlatInlineFragment {
                    text: text.clone(),
                    style,
                    link: link.map(str::to_string),
                    link_run_id,
                });
            }
            InlineNode::Emphasis(children) => {
                let mut style = style.clone();
                style.emphasis = true;
                flatten_inline_nodes(out, children, &style, link, link_run_id, next_link_run_id);
            }
            InlineNode::Strong(children) => {
                let mut style = style.clone();
                style.strong = true;
                flatten_inline_nodes(out, children, &style, link, link_run_id, next_link_run_id);
            }
            InlineNode::Strikethrough(children) => {
                let mut style = style.clone();
                style.strikethrough = true;
                flatten_inline_nodes(out, children, &style, link, link_run_id, next_link_run_id);
            }
            InlineNode::Mark(children)
            | InlineNode::Subscript(children)
            | InlineNode::Superscript(children) => {
                flatten_inline_nodes(out, children, style, link, link_run_id, next_link_run_id);
            }
            InlineNode::Link { children, url, .. } => {
                let current_link_run_id = *next_link_run_id;
                *next_link_run_id += 1;
                flatten_inline_nodes(
                    out,
                    children,
                    style,
                    Some(url),
                    Some(current_link_run_id),
                    next_link_run_id,
                );
            }
            InlineNode::Image { alt, url, .. } => {
                let text = if alt.is_empty() {
                    url.clone()
                } else {
                    alt.clone()
                };
                out.push(FlatInlineFragment {
                    text,
                    style: style.clone(),
                    link: Some(url.clone()),
                    link_run_id: Some(*next_link_run_id),
                });
                *next_link_run_id += 1;
            }
            InlineNode::FootnoteReference(id) => {
                out.push(FlatInlineFragment {
                    text: format!("[{id}]"),
                    style: style.clone(),
                    link: None,
                    link_run_id: None,
                });
            }
        }
    }
}

#[cfg(test)]
#[path = "inline_tests.rs"]
mod tests;

#[allow(dead_code)]
pub(crate) fn render_inline_formatted(
    ui: &mut egui::Ui,
    content: &str,
    wrap_width: f32,
    base_style: InlineBaseStyle,
) {
    let segments = parse_inline_formatting(content);

    if segments.len() == 1 {
        let span = &segments[0];
        if !span.is_code {
            let mut rich = egui::RichText::new(span.text.as_str());
            let is_bold = span.is_bold || base_style == InlineBaseStyle::Strong;
            if is_bold {
                rich = rich.strong();
            }
            ui.add(egui::Label::new(rich).wrap_mode(egui::TextWrapMode::Wrap));
            return;
        }
    }

    let style = ui.style().clone();
    let text_color = style.visuals.text_color();
    let strong_color = style.visuals.strong_text_color();
    let mut job = egui::text::LayoutJob::default();

    for span in segments {
        if span.text.is_empty() {
            continue;
        }

        if span.is_code {
            egui::RichText::new(span.text)
                .code()
                .color(text_color)
                .append_to(
                    &mut job,
                    &style,
                    egui::FontSelection::Default,
                    ui.text_valign(),
                );
            continue;
        }

        let is_bold = span.is_bold || base_style == InlineBaseStyle::Strong;
        let mut rich = egui::RichText::new(span.text);
        if is_bold {
            rich = rich.strong().color(strong_color);
        } else {
            rich = rich.color(text_color);
        }
        rich.append_to(
            &mut job,
            &style,
            egui::FontSelection::Default,
            ui.text_valign(),
        );
    }

    job.wrap = egui::text::TextWrapping::wrap_at_width(wrap_width.max(0.0));

    let galley = ui.fonts(|fonts| fonts.layout_job(job));
    ui.label(galley);
}

#[allow(dead_code)]
pub(crate) fn layout_job_from_inline_markdown(
    ui: &egui::Ui,
    content: &str,
    wrap_width: f32,
    render_style: InlineRenderStyle,
) -> egui::text::LayoutJob {
    let mut job = egui::text::LayoutJob::default();
    let style = ui.style();
    let visuals = &style.visuals;
    let text_color = if render_style.quote {
        visuals.weak_text_color()
    } else {
        visuals.text_color()
    };
    let strong_color = if render_style.quote {
        visuals.weak_text_color()
    } else {
        visuals.strong_text_color()
    };
    let link_color = visuals.hyperlink_color;
    let code_bg = visuals.code_bg_color;
    let code_size = render_style
        .heading
        .map(|level| heading_font_size(ui, level))
        .unwrap_or_else(|| egui::TextStyle::Monospace.resolve(style).size);
    let code_font = egui::FontId::monospace(code_size);
    let body_font = egui::TextStyle::Body.resolve(style);

    let mut options = Options::empty();
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_HEADING_ATTRIBUTES);

    let mut strong_depth = usize::from(render_style.base == InlineBaseStyle::Strong);
    let mut emphasis_depth = 0usize;
    let mut strikethrough_depth = 0usize;
    let mut link_destinations: Vec<String> = Vec::new();

    for event in Parser::new_ext(content, options) {
        match event {
            Event::Start(tag) => match tag {
                Tag::Strong => strong_depth += 1,
                Tag::Emphasis => emphasis_depth += 1,
                Tag::Strikethrough => strikethrough_depth += 1,
                Tag::Link { dest_url, .. } => link_destinations.push(dest_url.to_string()),
                _ => {}
            },
            Event::End(tag) => match tag {
                TagEnd::Strong => strong_depth = strong_depth.saturating_sub(1),
                TagEnd::Emphasis => emphasis_depth = emphasis_depth.saturating_sub(1),
                TagEnd::Strikethrough => {
                    strikethrough_depth = strikethrough_depth.saturating_sub(1)
                }
                TagEnd::Link => {
                    link_destinations.pop();
                }
                _ => {}
            },
            Event::Text(text) => {
                append_inline_text(
                    &mut job,
                    &text,
                    body_font.clone(),
                    strong_depth > 0,
                    emphasis_depth > 0,
                    strikethrough_depth > 0,
                    link_destinations.last().is_some(),
                    text_color,
                    strong_color,
                    link_color,
                    None,
                );
            }
            Event::Code(code) => {
                append_inline_text(
                    &mut job,
                    &code,
                    code_font.clone(),
                    true,
                    false,
                    false,
                    false,
                    text_color,
                    strong_color,
                    link_color,
                    Some(code_bg),
                );
            }
            Event::SoftBreak => job.append(" ", 0.0, egui::TextFormat::default()),
            Event::HardBreak => job.append("\n", 0.0, egui::TextFormat::default()),
            Event::Rule => job.append("\n---\n", 0.0, egui::TextFormat::default()),
            _ => {}
        }
    }

    job.wrap = egui::text::TextWrapping::wrap_at_width(wrap_width.max(0.0));
    job
}

#[allow(dead_code, clippy::too_many_arguments)]
pub(crate) fn append_inline_text(
    job: &mut egui::text::LayoutJob,
    text: &str,
    font_id: egui::FontId,
    strong: bool,
    emphasis: bool,
    strikethrough: bool,
    link: bool,
    text_color: egui::Color32,
    strong_color: egui::Color32,
    link_color: egui::Color32,
    background: Option<egui::Color32>,
) {
    if text.is_empty() {
        return;
    }

    let mut format = egui::TextFormat {
        font_id,
        color: if link {
            link_color
        } else if strong {
            strong_color
        } else {
            text_color
        },
        ..Default::default()
    };

    if strong {
        format.font_id.size += 0.5;
    }
    if emphasis {
        format.italics = true;
    }
    if strikethrough {
        format.strikethrough = egui::Stroke::new(1.0, format.color);
    }
    if link {
        format.underline = egui::Stroke::new(1.0, format.color);
    }
    if let Some(background) = background {
        format.background = background;
    }

    job.append(text, 0.0, format);
}
