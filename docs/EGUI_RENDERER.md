# egui renderer documentation

This document describes the egui rendering behavior of `the-markdown-commons` crate when using the `egui` feature flag.

## Feature flag

Enable the egui renderer with:

```toml
[dependencies]
tmc = { version = "0.1", features = ["egui"] }
```

## Basic usage

```rust
use tmc::{egui::{render_markdown, MarkdownCache}, Ui};

fn render_markdown_content(ui: &mut Ui, cache: &mut MarkdownCache, text: &str) {
    render_markdown(ui, cache, text, "my-content");
}
```

## `MarkdownCache`

`MarkdownCache` wraps `DocumentCache` to reuse parsed documents across renders:

```rust
use tmc::egui::MarkdownCache;

let mut cache = MarkdownCache::default();
// reuse cache across frames
```

## `MarkdownRenderOptions`

```rust
use tmc::egui::{render_markdown_with_options, MarkdownRenderOptions};

render_markdown_with_options(
    ui,
    cache,
    text,
    id_salt,
    &MarkdownRenderOptions {
        default_code_language: Some("sh"),
        code_language_overrides: &[("text", "sh"), ("txt", "sh")],
    },
);
```

| Option | Description |
|--------|-------------|
| `default_code_language` | Language for fenced code blocks without an info string |
| `code_language_overrides` | Map specific language tags to different highlight languages |

## `MarkdownSegment` and `parse_markdown_segments`

For custom table rendering, split content into segments:

```rust
use tmc::egui::{parse_markdown_segments, MarkdownSegment};

let segments = parse_markdown_segments("Text\n\n| a | b |\n|---|---|\n");
for segment in segments {
    match segment {
        MarkdownSegment::Text(text) => { /* render text */ }
        MarkdownSegment::Table(table) => { /* render table */ }
    }
}
```

`MarkdownTable` contains raw string cells (not inline-parsed content):

```rust
pub struct MarkdownTable {
    pub headers: Vec<String>,
    pub alignments: Vec<Alignment>,  // currently unused in rendering
    pub rows: Vec<Vec<String>>,
}
```

## Rendering behavior by block type

### Paragraphs

Paragraphs wrap within the available width using egui's label widget. Soft breaks become spaces; hard breaks use a `<br>` equivalent via explicit newline in the label text.

### Headings

Headings render with decreasing font sizes (egui's `TextStyle::Heading1` through `Heading6`). Inline formatting (code, emphasis, links) is supported within headings.

### Lists

- **Unordered lists**: bullet points with 4-space indent for nesting
- **Ordered lists**: numeric markers, correct double-digit alignment
- **Task lists**: checkbox icon (checked/unchecked) followed by text; checkboxes are display-only
- Lazy continuation works: paragraphs not indented enough still continue the list item

### Tables

Tables render with header row in bold. Cell content wraps within the column width. Alignment hints (`Alignment`) are stored in `MarkdownTable` but not currently applied to egui layout.

### Code blocks

Fenced code blocks use `egui_extras` syntax highlighting with syntect. Language is extracted from the info string. Without a language, `default_code_language` is used if set.

### Blockquotes

Blockquotes render with a vertical bar on the left and indented content.

### Inline formatting

Supported inline elements:
- `**strong**` → bold
- `*emphasis*` → italic
- `` `code` `` → monospace font
- `[link](url)` → link-colored label with an in-row hover underline overlay
- `~~strikethrough~~` → strikethrough style
- `==mark==` → highlight color
- `~subscript~` → smaller text below baseline
- `^superscript^` → smaller text above baseline

## Known limitations

1. **Links use the default URL handler** — the egui renderer opens links directly and does not yet expose a custom navigation callback
2. **Table alignment** — alignment hints in `MarkdownTable.alignments` are not applied to egui layout
3. **Images** — images are not yet rendered visually; they appear as alt text or placeholder
Fenced code blocks provide a copy-to-clipboard button. The headless test
`test_headless_code_block_copy_button_copies_text` covers its clipboard action.

## Snapshot tests

The egui renderer has headless snapshot tests in `src/egui/headless_tests.rs`. These validate layout and styling without requiring visual UI inspection. These tests require Rust 1.86+ due to egui dependency changes.
