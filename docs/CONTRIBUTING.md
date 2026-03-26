# Contributing to the-markdown-commons

This guide explains how to add features, tests, and documentation to the `the-markdown-commons` crate.

## Repository structure

```
src/
  lib.rs           — crate entry point and public exports
  document.rs      — block-level document model and parser
  inline.rs        — inline node parser
  html.rs          — HTML renderer with HtmlOptions
  parse_util.rs    — shared parsing utilities
  common.rs        — shared types (Alignment, etc.)
  egui.rs          — egui module root
  egui/
    render.rs      — main egui rendering entry points
    inline.rs      — inline rendering for egui
    segments.rs    — markdown segment parsing (text vs table)
    lists.rs       — list rendering
    style.rs       — styling constants
    headless_tests.rs — headless snapshot tests
    parse.rs       — parser tests for egui-specific behavior

tests/
  public_api.rs    — public API compile tests
  public_api_egui.rs — egui public API tests (feature-gated)
  egui_auto_ui.rs  — X11 screenshot regression tests
  egui_snapshots.rs — snapshot comparison tests

docs/
  SECURITY.md      — security documentation
  PARSER_BEHAVIOR.md
  HTML_RENDERER.md
  EGUI_RENDERER.md
  CACHE.md
```

## Adding a new block or inline feature

1. **Add parser support** in `src/document.rs` or `src/inline.rs`
2. **Add tests** in the `#[cfg(test)]` module of the relevant file
3. **Add HTML renderer support** in `src/html/block.rs` or `src/html/inline.rs`
4. **Add egui renderer support** in `src/egui/` submodules
5. **Update documentation** in `docs/PARSER_BEHAVIOR.md`, `docs/HTML_RENDERER.md`, and `docs/EGUI_RENDERER.md`

## Test categories

### Unit tests

Parser and renderer unit tests live in `#[cfg(test)]` modules within each source file. Run with:

```bash
cargo test --release
```

### Public API tests

Tests in `tests/public_api.rs` verify that exported functions and types remain usable. These run in default mode.

### HTML renderer tests

Tests in `src/html/tests.rs` cover HTML output for all supported syntax. 133 tests.

### Inline parser tests

Tests in `src/inline.rs` `#[cfg(test)]` module cover inline parsing.

### Document cache tests

Tests in `src/document.rs` `document::cache::tests` module cover eviction and cache behavior.

### egui headless tests

Tests in `src/egui/headless_tests.rs` validate egui rendering without visual inspection. These run with the `egui` feature on the current stable toolchain.

### Snapshot tests

Snapshot tests use `insta`. Run with:

```bash
cargo test --release --features egui -- --test-threads=1
```

Update snapshots with:

```bash
INSTA_UPDATE=always cargo test --features egui
```

### Security regression tests

Tests in `src/html/tests.rs` covering raw HTML, XSS payloads, dangerous URLs, and escaping behavior.

## Adding documentation

Documentation files are in `docs/`. Write in Markdown with examples that compile.

When documenting parser or renderer behavior:
- Include code examples that show the input and expected output
- Note any edge cases or known limitations
- Reference the tests that validate the behavior

## Adding benchmarks

Benchmarks live in `benches/parse.rs`. Run with:

```bash
cargo bench --release --bench parse
```

Benchmark fixtures should be in `benchmarks/corpus/` but may be ignored by `.gitignore` due to `.md` extension. Use a non-ignored extension or force-add with `git add -f`.

## Feature flags

| Flag | Enables | Notes |
|------|---------|-------|
| `default` | HTML only | Apply the embedding and URL policy described in SECURITY.md |
| `egui` | egui renderer | Adds egui, egui_extras, pulldown-cmark dependencies |
| `auto-ui-tests` | X11 screenshot tests | Requires `DISPLAY`, `xdotool`, `wmctrl`, `import`, `convert`, and `identify`; tests skip when unavailable |

## Running all tests

```bash
cargo test --release
cargo test --release --features egui
cargo test --release --all-features --lib --tests --bins -- --test-threads=1
```

## CI checklist

Before submitting a PR, verify:
- `cargo fmt` passes
- `cargo clippy --release -- -D warnings` passes
- `cargo test --release` passes (133+ unit tests)
- `cargo test --release --features egui` passes (if egui changes)
- `cargo test --release --all-features --lib --tests --bins -- --test-threads=1` passes when feature-gated or X11-related code changes
- New public API has doc comments and examples
- New behavior has corresponding tests
- Documentation is updated for behavior changes
