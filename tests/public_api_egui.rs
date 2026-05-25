#![cfg(feature = "egui")]

use tmc::{render_markdown, MarkdownCache, MarkdownSegment, MarkdownTable};

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

#[test]
fn root_egui_api_renders_headlessly() {
    let ctx = egui::Context::default();
    let mut cache = MarkdownCache::default();

    let output = ctx.run(headless_input(0.0), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            render_markdown(
                ui,
                &mut cache,
                "# Title\n\n[link](https://example.com)\n\n```rs\nfn main() {}\n```",
                "public_api",
            );
        });
    });

    assert!(!output.shapes.is_empty());
}

#[test]
fn module_egui_api_renders_headlessly() {
    let ctx = egui::Context::default();
    let mut cache = tmc::egui::MarkdownCache::default();

    let output = ctx.run(headless_input(0.0), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            tmc::egui::render_markdown(
                ui,
                &mut cache,
                "> quote\n\n| A | B |\n| --- | --- |\n| 1 | 2 |",
                "module_api",
            );
        });
    });

    assert!(!output.shapes.is_empty());
}

#[test]
fn module_segment_parser_splits_tables_from_text() {
    let segments = tmc::egui::parse_markdown_segments(
        "Before\n\n| A | B |\n| --- | --- |\n| 1 | 2 |\n\nAfter",
    );

    assert_eq!(segments.len(), 3);
    assert!(matches!(&segments[0], MarkdownSegment::Text(text) if text.contains("Before")));
    assert!(matches!(&segments[1], MarkdownSegment::Table(table) if table.rows.len() == 1));
    assert!(matches!(&segments[2], MarkdownSegment::Text(text) if text.contains("After")));
}

#[test]
fn public_table_and_segment_types_are_constructible() {
    let table = MarkdownTable {
        headers: vec!["Feature".into(), "Value".into()],
        alignments: vec![tmc::Alignment::Left, tmc::Alignment::Right],
        rows: vec![vec!["speed".into(), "fast".into()]],
    };

    let segment = MarkdownSegment::Table(table.clone());

    match segment {
        MarkdownSegment::Table(parsed) => {
            assert_eq!(parsed.headers, table.headers);
            assert_eq!(parsed.rows, table.rows);
        }
        MarkdownSegment::Text(_) => panic!("expected table segment"),
    }
}
