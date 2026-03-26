# Parser behavior documentation

This document describes the markdown syntax supported by `the-markdown-commons` crate and notable edge case behavior.

## Supported block syntax

### Headings

ATX headings (`#` through `######`) are supported:

```markdown
# Level 1
## Level 2
### Level 3
#### Level 4
##### Level 5
###### Level 6
```

- Heading text may contain inline formatting (emphasis, strong, code, links)
- Trailing `#` characters are stripped: `### text ###` → `<h3>text</h3>`
- Custom IDs via `{#id}` syntax: `# Heading {#custom-id}` → `<h1 id="custom-id">Heading</h1>`
- Invalid: level 7 or higher renders as plain text

Setext headings are supported for levels 1 and 2:

```markdown
Heading
=======

Heading
-------
```

### Paragraphs

Paragraphs are separated by blank lines. Soft breaks (`\n` in source) render as a single space. Hard breaks require two trailing spaces or a backslash.

### Code blocks

**Fenced code blocks:**

```markdown
```javascript
const x = 1;
```
```

- Opening fence must be 3+ backticks or tildes
- Closing fence must be at least as long as opening fence
- Info string after opening fence is stored as language
- Content is literal — no markdown processing inside code blocks
- Mismatched closing fence lengths prevent the block from closing

**Indented code blocks:**

Lines indented 4+ spaces (or a tab) start an indented code block. Content is literal.

### Lists

**Unordered lists** support `-`, `*`, and `+` markers:

```markdown
- item one
- item two
  - nested item
```

- 4-space indentation creates nested lists
- Lazy continuation: a paragraph not indented enough still continues the list item

**Ordered lists** support numeric markers:

```markdown
1. First
2. Second
10. Tenth (works)
```

- Double-digit items render correctly
- Repeated `1.` syntax (common in some tools) is treated as continuation of the same list
- `start` attribute on `<ol>` handles non-1 starts

**Task lists:**

```markdown
- [ ] unchecked
- [x] checked
- [X] checked (uppercase)
```

### Tables

All outer-pipe shapes are supported:

```markdown
| a | b | c |     (leading and trailing pipes)
a | b | c             (no outer pipes)
| a | b | c           (leading pipe only)
a | b | c |           (trailing pipe only)
```

- Alignment row uses `---`, `:---`, `---:`, or `:---:`
- Cells may contain inline formatting
- Escaped pipes (`\|`) inside cells are preserved
- Row length mismatch is tolerated — missing cells are empty

### Blockquotes

```markdown
> quote
> > nested quote
```

- May contain other blocks (lists, code blocks, nested blockquotes)
- Lazy continuation: paragraphs starting with `>` but not indented continue the blockquote

### Horizontal rules

Three or more of `-`, `*`, or `_` with optional spaces:

```markdown
---
***
___
```

- Must be the only content on the line (after trimming)

### Definition lists

```markdown
Term
: definition
```

- Term line followed by lines starting with `: `
- Multiple definitions per term supported

## Supported inline syntax

### Emphasis and strong

- `*text*` — emphasis
- `_text_` — emphasis (with word-boundary rules for underscore)
- `**text**` — strong
- `__text__` — strong (with word-boundary rules)
- `***text***` — emphasis containing strong

Intraword underscores stay literal: `foo_bar_baz` is not emphasis.

### Code spans

- `` `code` `` — single backticks
- ``` ``code`` ``` — double backticks (inner backticks allowed)

### Links

**Inline links:**

```markdown
[text](url "title")
[text](url)
```

**Reference links:**

```markdown
[text][label]
[text][] (shortcut)
[label]: url "title"
```

- Labels are case-insensitive
- Whitespace is normalized in labels

**Autolinks:**

```markdown
https://example.com
<https://example.com>
<mailto:user@example.com>
```

Bare URLs ending in balanced parentheses keep the parentheses: `https://example.com/path(tool)` preserves `(tool)`.

### Images

Same link syntax with `!` prefix: `![alt](url "title")`

### Strikethrough, mark, subscript, superscript

```markdown
~~strikethrough~~
==mark==
~sub~script
^super^script
```

- Subscript and superscript reject whitespace-only content

### Raw HTML

**Default (`allow_raw_html: true`):** HTML tags pass through:

```markdown
<strong>bold</strong>
```

**Safe mode (`allow_raw_html: false`):** HTML is escaped:

```markdown
<strong> → &lt;strong&gt;
```

### HTML entities

Entities like `&lt;`, `&gt;`, `&amp;`, `&#124;` are preserved in output (not decoded or double-encoded).

### Escaped characters

Backslash before these characters produces a literal: `\*`, `\_`, `\[`, `\]`, `\<`, `\>`, `\#`, `\|`, `\!`, `\``, `\\`

## Parser limitations and deviations from CommonMark

1. **No GFM table extension** — tables follow a more permissive syntax without emoji in alignment markers
2. **No fenced code block language alias resolution** — info strings are stored as-is
3. **Intraword underscores are literal** — `foo_bar` is not parsed as emphasis, even though some parsers treat it as such
4. **Definition lists** are a non-standard extension with `:` prefix for definitions
5. **Task list syntax** accepts both `- [ ]` and `* [ ]` as markers
6. **Entities are not decoded** — `&lt;` stays as `&lt;` in output, not as `<`

## Block-aware metadata extraction

Link reference definitions (`[label]: url`) and footnote definitions (`[^id]: text`) are extracted during the block parsing pass, not after. This means:

- Definitions inside fenced code blocks are **preserved as code content**, not extracted as metadata
- Definitions inside indented code blocks are **preserved as code content**, not extracted as metadata

This prevents false metadata from code examples being treated as actual document references.

## Malformed input handling

The parser degrades gracefully to literal text rather than panicking or producing corrupt output:

- Unmatched `[` or `]` stays as literal text
- Unclosed code spans stay as literal backticks
- Invalid heading syntax renders as plain text
- Malformed tables render as plain text