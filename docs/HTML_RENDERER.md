# HTML renderer documentation

This document describes the HTML output produced by `the-markdown-commons` crate when using the `parse_markdown`, `parse_markdown_with_options`, or related functions.

## Output structure

HTML is produced block-by-block with newline-terminated tags:

```html
<h1>Title</h1>
<p>Paragraph text.</p>
<ul>
<li>Item</li>
</ul>
```

## `HtmlOptions`

| Option | Default | Description |
|--------|---------|-------------|
| `link_target_blank` | `true` | Adds `target="_blank"` to links |
| `allow_raw_html` | `true` | Passes raw HTML through unescaped |

### Raw HTML control for untrusted input

```rust
HtmlOptions {
    allow_raw_html: false,
    link_target_blank: true,  // or false, depending on your needs
}
```

Disabling raw HTML does not validate link or image URL schemes. Apply an
independent URL allowlist or an embedding sanitizer; see [SECURITY.md](SECURITY.md).

## HTML escaping

The renderer escapes all HTML-sensitive characters in:
- Link destinations (`href`)
- Image destinations (`src`) and alt text
- Link titles
- Code block content (fenced and indented)
- Inline code spans
- Table cell content

Escaped characters: `&` → `&amp;`, `<` → `&lt;`, `>` → `&gt;`, `"` → `&quot;`

## Block rendering

### Headings

```markdown
# Heading {#id}
```
```html
<h1 id="id">Heading</h1>
```

Custom IDs use `{#id}` syntax. IDs are HTML-escaped.

### Paragraphs

```markdown
Line one
line two
```
```html
<p>Line one line two</p>
```

A single newline within a paragraph becomes a space. Two trailing spaces or a backslash at line end produces a `<br>`:

```markdown
Line with break\

Second line
```
```html
<p>Line with break<br>
Second line</p>
```

### Code blocks

```markdown
```javascript
const x = 1;
```
```
```html
<pre><code class="language-javascript">const x = 1;</code></pre>
```

The language class uses `language-` prefix. Empty info strings produce no class.

### Tables

```html
<table>
<thead>
<tr>
<th style="text-align: left;">Name</th>
...
</tr>
</thead>
<tbody>
<tr>
<td>Value</td>
...
</tr>
</tbody>
</table>
```

Alignment uses inline `style` attributes: `text-align: left`, `text-align: center`, `text-align: right`.

### Lists

**Unordered lists:**

```html
<ul>
<li>Item</li>
<li>Item with
<br>
continuation</li>
</ul>
```

**Ordered lists:**

```html
<ol>
<li>First</li>
</ol>
<ol start="5">
<li>Fifth</li>
</ol>
```

Non-1 start values use `start` attribute. Repeated `1.` markers are merged into a single list.

**Task lists:**

```html
<ul class="task-list">
<li class="task-list-item"><label><input type="checkbox" disabled> <span>Text</span></label></li>
<li class="task-list-item"><label><input type="checkbox" disabled checked> <span>Text</span></label></li>
</ul>
```

Checkboxes are `disabled` (not interactive). Both checked and unchecked states render correctly.

### Blockquotes

```html
<blockquote>
<p>Quoted text</p>
</blockquote>
```

### Definition lists

```html
<dl>
<dt>Term</dt>
<dd>Definition</dd>
</dl>
```

### Horizontal rules

```html
<hr>
```

### Footnotes

When `parse_markdown` (full document) is used, a footnote section is appended:

```html
<section class="footnotes">
<hr>
<ol>
<li id="fn:1">Footnote text<a href="#fnref:1" class="footnote-backref">↩</a></li>
</ol>
</section>
```

Footnote references within content use the same format.

`parse_markdown_fragment` does **not** produce a footnote section.

## Inline rendering

### Links

```html
<a href="https://example.com" target="_blank">Link text</a>
```

`target="_blank"` is added when `link_target_blank: true`. Links are escaped in both href and body.

### Images

```html
<img src="https://example.com/image.png" alt="Alt text" title="Optional title">
```

Both src and alt are HTML-escaped.

### Emphasis and strong

```html
<em>emphasized</em>
<strong>strong</strong>
<strong><em>combined</em></strong>
```

### Code spans

```html
<code>inline code</code>
```

Content is HTML-escaped.

### Strikethrough, mark, subscript, superscript

```html
<del>strikethrough</del>
<mark>mark</mark>
<sub>sub</sub>
<sup>sup</sup>
```

### Raw HTML

When `allow_raw_html: true`, raw HTML tags pass through unchanged. When `false`, they are escaped:

```html
&lt;script&gt;alert(1)&lt;/script&gt;
```

## Security notes

- **Default `allow_raw_html: true`** is not safe for untrusted input
- `javascript:` and other dangerous URL schemes are not filtered; validate destinations independently of HTML escaping
- Add any required `rel="noopener noreferrer"` attributes in the embedding application or an HTML post-processor; CSS cannot set those attributes

## Full vs. fragment rendering

| Function | Footnotes | Use case |
|----------|-----------|----------|
| `parse_markdown` | Yes | Full documents |
| `parse_markdown_fragment` | No | Embedding snippets |

Both use the same block and inline parsing.