use super::{
    build_inline_layout, collect_link_spans, flatten_inline_nodes, hovered_link_row_ranges,
    render_selectable_inline, FlatInlineFragment, InlineRenderStyle, WidgetStyle,
};
use crate::inline::{parse_inline_nodes, LinkDefinitions};

fn flatten(content: &str) -> Vec<FlatInlineFragment> {
    let nodes = parse_inline_nodes(content, &LinkDefinitions::default());
    let mut fragments = Vec::new();
    let mut next_link_run_id = 0usize;
    flatten_inline_nodes(
        &mut fragments,
        &nodes,
        &WidgetStyle {
            heading: InlineRenderStyle::normal().heading,
            strong: false,
            quote: false,
            ..Default::default()
        },
        None,
        None,
        &mut next_link_run_id,
    );
    fragments
}

fn covered_row_text(
    galley: &egui::Galley,
    row_index: usize,
    column_range: std::ops::Range<usize>,
) -> String {
    galley.rows[row_index]
        .text()
        .chars()
        .skip(column_range.start)
        .take(column_range.end - column_range.start)
        .collect()
}

fn painted_bounds(shapes: Vec<egui::epaint::ClippedPrimitive>) -> egui::Rect {
    let mut bounds = egui::Rect::NOTHING;

    for clipped in shapes {
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

fn inline_hover_pointer_pos(
    ctx: &egui::Context,
    width: f32,
    fragments: &[FlatInlineFragment],
    label_text: &str,
) -> egui::Pos2 {
    let mut pointer_pos = egui::Pos2::ZERO;

    let _ = ctx.run(Default::default(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.set_width(width);
            ui.set_min_width(width);
            ui.set_max_width(width);

            let inline = build_inline_layout(ui, fragments);
            let label = egui::Label::new(inline.job.clone())
                .selectable(true)
                .wrap_mode(egui::TextWrapMode::Wrap)
                .sense(egui::Sense::click());
            let (galley_pos, galley, _) = label.layout_in_ui(ui);
            let hovered_link = inline
                .links
                .iter()
                .find(|link| inline.job.text[link.byte_range.clone()].contains(label_text))
                .expect("expected hovered link");
            let row_range = hovered_link_row_ranges(&galley, hovered_link)
                .into_iter()
                .next()
                .expect("expected hovered row range");
            let row = &galley.rows[row_range.row_index];
            let start_x = row.x_offset(row_range.column_range.start);
            let end_x = row.x_offset(row_range.column_range.end);
            pointer_pos =
                galley_pos + egui::vec2((start_x + end_x) * 0.5, (row.min_y() + row.max_y()) * 0.5);
        });
    });

    pointer_pos
}

fn render_inline_response_and_bounds(
    ctx: &egui::Context,
    width: f32,
    fragments: &[FlatInlineFragment],
    pointer_pos: Option<egui::Pos2>,
) -> (egui::Rect, egui::Rect) {
    let mut response_rect = egui::Rect::NOTHING;
    let output = ctx.run(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(800.0, 600.0),
            )),
            events: pointer_pos
                .map(|pos| vec![egui::Event::PointerMoved(pos)])
                .unwrap_or_default(),
            ..Default::default()
        },
        |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                ui.set_width(width);
                ui.set_min_width(width);
                ui.set_max_width(width);
                response_rect =
                    render_selectable_inline(ui, build_inline_layout(ui, fragments)).rect;
            });
        },
    );
    let tessellated = ctx.tessellate(output.shapes, output.pixels_per_point);

    (response_rect, painted_bounds(tessellated))
}

#[test]
fn adjacent_links_with_same_destination_keep_separate_runs() {
    let fragments = flatten("[first](https://example.com)[second](https://example.com)");

    let link_run_ids: Vec<_> = fragments
        .iter()
        .map(|fragment| fragment.link_run_id)
        .collect();

    assert_eq!(fragments.len(), 2);
    assert_eq!(link_run_ids, vec![Some(0), Some(1)]);
}

#[test]
fn link_followed_by_code_produces_separate_fragments() {
    let fragments = flatten("[link](https://example.com) `code`");
    assert!(
        fragments.len() >= 3,
        "expected at least link + space + code"
    );

    let link_frag = fragments.iter().find(|f| f.link.is_some()).unwrap();
    assert_eq!(link_frag.text, "link");
    assert_eq!(link_frag.link.as_deref(), Some("https://example.com"));

    let code_frag = fragments.iter().find(|f| f.style.code).unwrap();
    assert_eq!(code_frag.text, "code");
    assert!(code_frag.link.is_none());
}

#[test]
fn code_inside_link_inherits_link_destination() {
    let fragments = flatten("[`code_fn`](https://example.com)");
    assert_eq!(fragments.len(), 1);
    let frag = &fragments[0];
    assert!(frag.style.code);
    assert_eq!(frag.link.as_deref(), Some("https://example.com"));
}

#[test]
fn link_then_plain_then_code_all_present() {
    let fragments = flatten("[a](https://a.com) hello `x`");
    let texts: Vec<&str> = fragments.iter().map(|f| f.text.as_str()).collect();
    assert!(texts.contains(&"a"), "missing link text: {texts:?}");
    assert!(texts.contains(&"x"), "missing code text: {texts:?}");
    assert!(
        fragments.iter().any(|f| f.text.contains("hello")),
        "missing plain text: {texts:?}"
    );
}

#[test]
fn emphasis_inside_link_propagates_both_styles() {
    let fragments = flatten("[*italic*](https://example.com)");
    assert_eq!(fragments.len(), 1);
    let frag = &fragments[0];
    assert!(frag.style.emphasis);
    assert!(frag.link.is_some());
}

#[test]
fn strong_propagates_through_flatten() {
    let fragments = flatten("**bold text**");
    assert!(!fragments.is_empty());
    assert!(fragments[0].style.strong);
}

#[test]
fn strikethrough_propagates_through_flatten() {
    let fragments = flatten("~~struck~~");
    assert!(!fragments.is_empty());
    assert!(fragments[0].style.strikethrough);
}

#[test]
fn widget_style_default_is_not_code() {
    let style = WidgetStyle::default();
    assert!(!style.code);
    assert!(!style.strong);
    assert!(!style.emphasis);
}

#[test]
fn widget_style_code_flag_set_for_code_fragment() {
    let fragments = flatten("`hello`");
    assert_eq!(fragments.len(), 1);
    assert!(fragments[0].style.code);
}

#[test]
fn image_produces_link_fragment() {
    let fragments = flatten("![alt text](https://img.png)");
    assert_eq!(fragments.len(), 1);
    let frag = &fragments[0];
    assert_eq!(frag.text, "alt text");
    assert_eq!(frag.link.as_deref(), Some("https://img.png"));
}

#[test]
fn image_without_alt_uses_url_as_text() {
    let fragments = flatten("![](https://img.png)");
    assert_eq!(fragments.len(), 1);
    assert_eq!(fragments[0].text, "https://img.png");
}

#[test]
fn formatted_link_collapses_to_single_link_span() {
    let fragments = flatten("[**bold** `code`](https://example.com)");
    let spans = collect_link_spans(&fragments);

    assert_eq!(spans.len(), 1);
    assert_eq!(spans[0].destination, "https://example.com");
    assert_eq!(spans[0].char_range, 0.."bold code".chars().count());
}

#[test]
fn hovering_one_link_does_not_underline_later_plain_text() {
    let fragments = flatten(
        "The new docs like [session-worker-lifecycle-spec.md](https://example.com/session), \
[spool-session-contract-versioning.md](https://example.com/spool), \
[property-test-plan.md](https://example.com/property), \
[state-module-segmentation.md](https://example.com/state), and \
[evidence-gate.md](https://example.com/evidence) are not consumed by runtime code.",
    );
    let ctx = egui::Context::default();
    let mut hovered_text = String::new();

    let _ = ctx.run(Default::default(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.set_width(260.0);
            ui.set_min_width(260.0);
            ui.set_max_width(260.0);

            let inline = build_inline_layout(ui, &fragments);
            let hovered_link = inline
                .links
                .iter()
                .find(|link| {
                    inline.job.text[link.byte_range.clone()].contains("property-test-plan.md")
                })
                .expect("expected property-test-plan link");
            let galley = ui.fonts(|fonts| fonts.layout_job(inline.job.clone()));
            let row_ranges = hovered_link_row_ranges(&galley, hovered_link);

            hovered_text = row_ranges
                .iter()
                .map(|row_range| {
                    covered_row_text(&galley, row_range.row_index, row_range.column_range.clone())
                })
                .collect();
        });
    });

    assert!(
        hovered_text == "property-test-plan.md",
        "hover underline should stay scoped to the hovered link, got {hovered_text:?}"
    );
}

#[test]
fn wrapped_filename_links_cover_full_hovered_labels() {
    let fragments = flatten(
        "The parser changes are in \
[adapter_git_verifier.rs](https://example.com/verifier), with regressions in \
[adapter_git_verifier_tests.rs](https://example.com/tests).",
    );
    let ctx = egui::Context::default();
    let mut first_label = String::new();
    let mut second_label = String::new();
    let mut first_rows = 0usize;
    let mut second_rows = 0usize;

    let _ = ctx.run(Default::default(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.allocate_ui(egui::vec2(120.0, 200.0), |ui| {
                ui.set_width(120.0);
                ui.set_min_width(120.0);
                ui.set_max_width(120.0);

                let inline = build_inline_layout(ui, &fragments);
                let galley = ui.fonts(|fonts| fonts.layout_job(inline.job.clone()));
                let first_link = inline
                    .links
                    .iter()
                    .find(|link| {
                        inline.job.text[link.byte_range.clone()].contains("adapter_git_verifier.rs")
                    })
                    .expect("expected adapter_git_verifier.rs link");
                let second_link = inline
                    .links
                    .iter()
                    .find(|link| {
                        inline.job.text[link.byte_range.clone()]
                            .contains("adapter_git_verifier_tests.rs")
                    })
                    .expect("expected adapter_git_verifier_tests.rs link");
                let first_ranges = hovered_link_row_ranges(&galley, first_link);
                let second_ranges = hovered_link_row_ranges(&galley, second_link);

                first_rows = first_ranges.len();
                second_rows = second_ranges.len();
                first_label = first_ranges
                    .iter()
                    .map(|row_range| {
                        covered_row_text(
                            &galley,
                            row_range.row_index,
                            row_range.column_range.clone(),
                        )
                    })
                    .collect();
                second_label = second_ranges
                    .iter()
                    .map(|row_range| {
                        covered_row_text(
                            &galley,
                            row_range.row_index,
                            row_range.column_range.clone(),
                        )
                    })
                    .collect();
            });
        });
    });

    assert_eq!(first_label, "adapter_git_verifier.rs");
    assert_eq!(second_label, "adapter_git_verifier_tests.rs");
    assert!(first_rows >= 2, "expected first filename link to wrap");
    assert!(second_rows >= 2, "expected second filename link to wrap");
}

#[test]
fn hovering_link_does_not_change_inline_layout_bounds() {
    let fragments = flatten(
        "The new docs like [session-worker-lifecycle-spec.md](https://example.com/session), \
[spool-session-contract-versioning.md](https://example.com/spool), \
[property-test-plan.md](https://example.com/property), and \
[evidence-gate.md](https://example.com/evidence) stay in place while hovered.",
    );
    let width = 260.0;
    let idle_ctx = egui::Context::default();
    let hover_ctx = egui::Context::default();
    let pointer_pos =
        inline_hover_pointer_pos(&hover_ctx, width, &fragments, "property-test-plan.md");

    let (idle_rect, idle_bounds) =
        render_inline_response_and_bounds(&idle_ctx, width, &fragments, None);
    let (hover_rect, hover_bounds) =
        render_inline_response_and_bounds(&hover_ctx, width, &fragments, Some(pointer_pos));

    assert_eq!(
        idle_rect, hover_rect,
        "hover should not relayout the widget"
    );
    assert!(
        (hover_bounds.width() - idle_bounds.width()).abs() <= 0.01,
        "hover should not widen painted bounds: idle={idle_bounds:?} hover={hover_bounds:?}"
    );
    assert!(
        (hover_bounds.height() - idle_bounds.height()).abs() <= 0.01,
        "hover should not change painted height: idle={idle_bounds:?} hover={hover_bounds:?}"
    );
}

#[test]
fn to_richtext_code_applies_code_method() {
    let style = WidgetStyle {
        code: true,
        ..Default::default()
    };
    assert!(style.code, "code flag must be true for code fragments");
}

#[test]
fn code_fragment_style_matches_expected_values() {
    let fragments = flatten("plain `code_word` more");
    let code_frag = fragments.iter().find(|f| f.style.code).unwrap();
    assert_eq!(code_frag.text, "code_word");
    assert!(!code_frag.style.strong);
    assert!(!code_frag.style.emphasis);
    assert!(!code_frag.style.strikethrough);
    assert!(code_frag.link.is_none());
}

#[test]
fn plain_text_fragment_is_not_code() {
    let fragments = flatten("just plain text");
    for frag in &fragments {
        assert!(
            !frag.style.code,
            "plain text should not be code: {:?}",
            frag.text
        );
    }
}
