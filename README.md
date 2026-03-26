# the-markdown-commons

A Rust crate for parsing markdown into a shared typed document model, with renderers for both HTML and [egui](https://github.com/emilk/egui).

`tmc::parse_document` is the primary typed API. The `parse_markdown` helpers are convenience HTML wrappers on top of that shared parse model.

## What is here

- `src/html.rs`: a pure Rust markdown-to-HTML renderer
- `src/egui.rs`: the `egui` module root with internal renderer submodules under `src/egui/`
- `src/lib.rs`: the crate entry point and feature-gated module exports

## Current implementation status

- The crate has a shared typed parse model:
  - `tmc::MarkdownDocument` for block-level structure
  - `tmc::InlineNode` for inline structure
  - `tmc::InlineContent` for stored source-plus-inline-node content inside document blocks
- The HTML renderer and the `egui` renderer both consume that shared parse layer.
- `tmc::DocumentCache` is public and reusable. The current API intentionally stays simple and returns cloned documents instead of borrowed handles.
- `tmc::MarkdownCache` reuses parsed document fragments on the `egui` path.
- The string-to-HTML helpers remain supported convenience APIs for callers that do not want to work with the typed document model directly.

## Features

- Default build: ships only the HTML parser
- `egui`: enables the `egui` renderer and its supporting dependencies
- `auto-ui-tests`: enables the optional `eframe` harness and X11 screenshot regression tests on top of `egui`
  - The tests compile in a clean checkout and skip themselves when `DISPLAY`, `xdotool`, `wmctrl`, `import`, `convert`, or `identify` are unavailable

## Quick start

```rust
use tmc::parse_markdown;

let html = parse_markdown("# Hello\n\nThis is **markdown**.");
assert!(html.contains("<h1>Hello</h1>"));
```

With the shared typed document model:

```rust
use tmc::{parse_document, Block};

let document = parse_document("# Hello\n\n- one\n- two");

assert!(matches!(document.blocks[0], Block::Heading(ref heading) if !heading.content.nodes.is_empty()));
assert!(matches!(document.blocks[1], Block::UnorderedList(_)));
```

With the shared inline parser:

```rust
use tmc::{parse_inline_nodes, InlineNode, LinkDefinitions};

let nodes = parse_inline_nodes(
    "A [link](https://example.com) and **bold** text",
    &LinkDefinitions::default(),
);

assert!(nodes.iter().any(|node| matches!(node, InlineNode::Link { .. })));
assert!(nodes.iter().any(|node| matches!(node, InlineNode::Strong(_))));
```

With reusable parsed-document caching:

```rust
use tmc::{parse_document_cached, parse_markdown_cached, DocumentCache};

let mut cache = DocumentCache::default();
let document = parse_document_cached(&mut cache, "# Hello\n\nParagraph");
let html = parse_markdown_cached(&mut cache, "# Hello\n\nParagraph");

assert!(!document.blocks.is_empty());
assert!(html.contains("<h1>Hello</h1>"));
```

The same cache also supports fragment parsing:

```rust
use tmc::{parse_document_fragment_cached, DocumentCache};

let mut cache = DocumentCache::default();
let fragment = parse_document_fragment_cached(&mut cache, "*item*");

assert!(!fragment.blocks.is_empty());
```

With explicit HTML parsing options:

```rust
use tmc::{parse_markdown_with_options, HtmlOptions};

let html = parse_markdown_with_options(
    "<em>raw</em> [Guide](https://example.com)",
    HtmlOptions {
        link_target_blank: false,
        allow_raw_html: false,
    },
);

assert!(html.contains("&lt;em&gt;raw&lt;/em&gt;"));
assert!(html.contains(r#"<a href="https://example.com">Guide</a>"#));
```

With the `egui` feature enabled:

```rust
use tmc::{render_markdown, MarkdownCache};

fn render(ui: &mut egui::Ui) {
    let mut cache = MarkdownCache::default();
    render_markdown(ui, &mut cache, "A [link](https://example.com)", "demo");
}
```

## Test commands

```bash
cargo fmt --check
cargo clippy --release --all-features -- \
  -A unused_variables \
  -A clippy::needless_range_loop \
  -A clippy::redundant_closure \
  -A clippy::manual_clamp \
  -D warnings
cargo test --release
cargo test --release --features egui
cargo test --release --all-features --lib --tests --bins -- --test-threads=1
cargo test --release --doc
cargo build --release --all-features
cargo bench --release --bench parse --no-run
cargo package --allow-dirty
```

## Security considerations

Raw HTML is enabled by default. Use `allow_raw_html: false` when raw markup is
not trusted, and independently validate link/image URL schemes. HTML escaping
does not establish a URL allowlist. See [Security](docs/SECURITY.md).

## Documentation

- [Parser behavior](docs/PARSER_BEHAVIOR.md)
- [HTML renderer](docs/HTML_RENDERER.md)
- [egui renderer](docs/EGUI_RENDERER.md)
- [Cache](docs/CACHE.md)
- [Known limitations](docs/KNOWN_LIMITATIONS.md)
- [Security](docs/SECURITY.md)
- [Contributing](docs/CONTRIBUTING.md)
- [Release checklist](docs/RELEASE_CHECKLIST.md)
- [Benchmarking](docs/SPEED_TESTING.md)

## License

First-party code is available under [ISC](LICENSE). Existing derived styling
notices are retained in [THIRD_PARTY_NOTICES](THIRD_PARTY_NOTICES).
