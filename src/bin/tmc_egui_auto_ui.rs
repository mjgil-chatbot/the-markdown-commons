#[cfg(feature = "auto-ui-tests")]
mod app {
    use std::env;
    use std::time::Duration;

    use tmc::{render_markdown, MarkdownCache};

    pub fn run() -> eframe::Result<()> {
        let title =
            env::var("TMC_EGUI_WINDOW_TITLE").unwrap_or_else(|_| "TMC Egui Auto UI".to_string());
        let markdown = env::var("TMC_EGUI_MARKDOWN")
            .map(|s| s.replace("\\n", "\n"))
            .unwrap_or_else(|_| {
                "# Auto UI Harness\n\nParagraph with **bold**, a [link](https://example.com), a table, and a code block.\n\n| Name | Value |\n| --- | --- |\n| Parser | active |\n| Renderer | ready |\n\n```json\n{\"ok\":true}\n```".to_string()
            });
        let width = env::var("TMC_EGUI_WINDOW_WIDTH")
            .ok()
            .and_then(|value| value.parse::<f32>().ok())
            .unwrap_or(560.0);
        let height = env::var("TMC_EGUI_WINDOW_HEIGHT")
            .ok()
            .and_then(|value| value.parse::<f32>().ok())
            .unwrap_or(760.0);

        let options = eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_title(title.clone())
                .with_inner_size([width, height]),
            ..Default::default()
        };

        eframe::run_native(
            &title,
            options,
            Box::new(move |_cc| {
                Ok(Box::new(HarnessApp {
                    markdown,
                    cache: MarkdownCache::default(),
                }))
            }),
        )
    }

    struct HarnessApp {
        markdown: String,
        cache: MarkdownCache,
    }

    impl eframe::App for HarnessApp {
        fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
            egui::CentralPanel::default().show(ctx, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    render_markdown(ui, &mut self.cache, &self.markdown, "auto_ui_harness");
                });
            });

            ctx.request_repaint_after(Duration::from_millis(16));
        }
    }
}

#[cfg(feature = "auto-ui-tests")]
fn main() -> eframe::Result<()> {
    app::run()
}

#[cfg(not(feature = "auto-ui-tests"))]
fn main() {
    eprintln!("Enable the `auto-ui-tests` feature to run this harness.");
}
