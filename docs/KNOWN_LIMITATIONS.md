# Known limitations

The parser implements its own markdown behavior and does not claim full
CommonMark or GFM conformance. See [PARSER_BEHAVIOR.md](PARSER_BEHAVIOR.md) for
supported syntax and [SECURITY.md](SECURITY.md) for untrusted input handling.

## egui tables

Reference-style links inside table cells are not resolved by the current egui
table path. The renderer reconstructs source text for table cells; inline links
are a workaround. The headless test `table_reference_link_in_egui` is explicitly
ignored with this limitation recorded in its reason.

## Cache ownership

`DocumentCache` returns owned documents, including a clone on cache hits. The
public API does not promise borrowed-document or zero-copy cache access.

## HTML policy

Raw HTML is enabled by default. Disabling it escapes raw markup but does not
filter dangerous URL schemes. Callers must apply their own URL and embedding
policy. The crate is not a complete HTML sanitizer.

## Display-dependent tests

The `auto-ui-tests` feature needs an X11 display and its documented external
utilities. Display tests skip when those dependencies are unavailable; a skip
is not evidence that screenshot behavior passed. Headless egui tests can run
without that display environment.
