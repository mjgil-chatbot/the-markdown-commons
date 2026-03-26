# Cache documentation

This document describes the caching behavior of `the-markdown-commons` crate.

## `DocumentCache`

`DocumentCache` stores parsed `MarkdownDocument` instances keyed by their input text. This avoids re-parsing the same markdown content.

```rust
use tmc::{DocumentCache, parse_document_cached};

let mut cache = DocumentCache::default();
let doc1 = parse_document_cached(&mut cache, "# Hello");
let doc2 = parse_document_cached(&mut cache, "# Hello");  // hits cache
```

### Cache organization

The cache maintains separate stores for:
- **Full documents** — parsed with `parse_document` or `parse_document_cached`
- **Fragments** — parsed with `parse_document_fragment` or `parse_document_fragment_cached`

Full documents and fragments are cached independently.

### LRU ordering

Cache entries maintain insertion order. When the cache grows too large, older entries are evicted.

### Eviction limits

| Limit | Default | Description |
|-------|---------|-------------|
| Entry count | 64 | Maximum documents per cache store |
| Byte size | 512 KB | Maximum total document bytes |

When either limit is exceeded, the oldest entries are evicted until the cache is within bounds.

### Cache methods

```rust
// Parse a full document (uses cache)
pub fn parse_document(&mut self, input: &str) -> MarkdownDocument;

// Parse a fragment (uses cache)
pub fn parse_document_fragment(&mut self, input: &str) -> MarkdownDocument;

// Clear all cached entries
pub fn clear(&mut self);
```

### `MarkdownCache` (egui)

`MarkdownCache` wraps `DocumentCache` for the egui rendering path:

```rust
use tmc::egui::{render_markdown, MarkdownCache};

let mut cache = MarkdownCache::default();
render_markdown(ui, &mut cache, "# Hello", "demo");
```

The inner `document_cache` field is public, so you can access `DocumentCache` methods directly:

```rust
cache.document_cache.clear();
```

## Cache usage example

For repeated rendering of the same content:

```rust
use tmc::{parse_markdown_cached, DocumentCache};

let mut cache = DocumentCache::default();

fn render(content: &str, cache: &mut DocumentCache) -> String {
    parse_markdown_cached(cache, content)
}

// First call parses, subsequent calls use cache
let html1 = render("# Title", &mut cache);
let html2 = render("# Title", &mut cache);  // fast cache hit
```

## Performance notes

- Cache lookup is O(1) (HashMap)
- Cloning the document on cache hit adds allocation cost
- The current API returns owned documents, not borrowed handles — this is intentional for API simplicity

## Clearing the cache

Call `clear()` to free memory or reset state:

```rust
cache.clear();
```

This removes all full documents and fragments from the cache.