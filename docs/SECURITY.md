# Security

`the-markdown-commons` is a markdown parser and renderer, not an HTML or URL
sanitizer. Treat provider and user-supplied markdown as untrusted input.

## HTML output

`HtmlOptions::default()` permits raw HTML. Set `allow_raw_html: false` to escape
raw HTML when it is not intentionally trusted:

```rust
use tmc::{parse_markdown_with_options, HtmlOptions};
let html = parse_markdown_with_options(input, HtmlOptions {
    allow_raw_html: false,
    link_target_blank: false,
});
```

This controls raw HTML only. Escaping an attribute does not make a dangerous
URL scheme safe. The current renderer does not enforce a protocol allowlist for
link and image URLs. Validate allowed protocols and destinations independently,
including `javascript:` and `data:` handling, or sanitize the rendered output
with an appropriate policy before embedding it in a trusted page.

`link_target_blank` controls new-window links. The renderer does not supply a
complete embedding policy; callers must decide `rel` attributes and Content
Security Policy. CSS cannot add security attributes or sanitize a URL.

## egui output

The egui renderer can emit link-opening and clipboard actions. Treat destinations
as untrusted and constrain navigation in the embedding application as needed.
Image loading and external resources depend on the host and enabled features.

## Regression coverage

`src/html/tests.rs` exercises raw HTML, attribute escaping, and malformed inputs.
The headless egui tests cover rendering and interaction behavior. These tests do
not establish full CommonMark compliance or comprehensive content sanitization.

Report suspected vulnerabilities privately to the repository maintainer, with
a minimal input, enabled features, and observed output.
