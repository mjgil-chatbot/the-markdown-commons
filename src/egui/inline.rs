use std::ops::Range;
use super::style::{heading_font_size, height_body};
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
#[derive(Clone)]
struct FlatInlineFragment {
    text: String,
    style: WidgetStyle,
    link: Option<String>,
    link_run_id: Option<usize>,
}
