#[cfg(feature = "egui")]
use std::borrow::Cow;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Alignment {
    Left,
    Center,
    Right,
}

pub fn fence_delimiter(line: &str) -> Option<char> {
    let trimmed = line.trim_start();
    if trimmed.starts_with("```") {
        Some('`')
    } else if trimmed.starts_with("~~~") {
        Some('~')
    } else {
        None
    }
}

#[cfg(feature = "egui")]
pub fn normalize_fenced_code_languages(content: &str) -> String {
    let mut out = String::with_capacity(content.len());
    let mut first = true;

    for line in content.lines() {
        if !first {
            out.push('\n');
        }
        first = false;

        if let Some(normalized) = normalize_fence_line(line) {
            out.push_str(&normalized);
        } else {
            out.push_str(line);
        }
    }

    if content.ends_with('\n') {
        out.push('\n');
    }

    out
}

#[cfg(feature = "egui")]
fn normalize_fence_line(line: &str) -> Option<String> {
    let trimmed = line.trim_start();
    let indent_len = line.len() - trimmed.len();

    let fence_char = trimmed.chars().next()?;
    if fence_char != '`' && fence_char != '~' {
        return None;
    }

    let fence_len = trimmed.chars().take_while(|c| *c == fence_char).count();
    if fence_len < 3 {
        return None;
    }

    let fence = &trimmed[..fence_len];
    let info = trimmed[fence_len..].trim();
    if info.is_empty() {
        return None;
    }

    let mut parts = info.splitn(2, char::is_whitespace);
    let lang = parts.next().unwrap_or("");
    let rest = parts.next().unwrap_or("").trim_start();

    let mapped = normalize_language_tag(lang);
    if mapped == lang {
        return None;
    }

    let mut normalized = String::new();
    normalized.push_str(&line[..indent_len]);
    normalized.push_str(fence);
    normalized.push_str(&mapped);
    if !rest.is_empty() {
        normalized.push(' ');
        normalized.push_str(rest);
    }

    Some(normalized)
}

#[cfg(feature = "egui")]
pub fn normalize_language_tag(lang: &str) -> Cow<'_, str> {
    if lang.bytes().all(|b| !b.is_ascii_uppercase()) {
        return match lang {
            "js" | "javascript" => Cow::Borrowed("js"),
            "ts" | "typescript" => Cow::Borrowed("ts"),
            "bash" | "sh" | "shell" | "zsh" => Cow::Borrowed("sh"),
            "py" | "python" => Cow::Borrowed("py"),
            "rs" | "rust" => Cow::Borrowed("rs"),
            "go" | "golang" => Cow::Borrowed("go"),
            "c" => Cow::Borrowed("c"),
            "cpp" | "c++" | "cxx" => Cow::Borrowed("cpp"),
            "cs" | "csharp" => Cow::Borrowed("cs"),
            "json" => Cow::Borrowed("json"),
            "yml" | "yaml" => Cow::Borrowed("yml"),
            "md" | "markdown" => Cow::Borrowed("md"),
            "toml" => Cow::Borrowed("toml"),
            "xml" => Cow::Borrowed("xml"),
            "html" => Cow::Borrowed("html"),
            "css" => Cow::Borrowed("css"),
            "scss" => Cow::Borrowed("scss"),
            "sql" => Cow::Borrowed("sql"),
            "diff" => Cow::Borrowed("diff"),
            _ => Cow::Borrowed(lang),
        };
    }

    let lower = lang.to_ascii_lowercase();
    match lower.as_str() {
        "js" | "javascript" => Cow::Borrowed("js"),
        "ts" | "typescript" => Cow::Borrowed("ts"),
        "bash" | "sh" | "shell" | "zsh" => Cow::Borrowed("sh"),
        "py" | "python" => Cow::Borrowed("py"),
        "rs" | "rust" => Cow::Borrowed("rs"),
        "go" | "golang" => Cow::Borrowed("go"),
        "c" => Cow::Borrowed("c"),
        "cpp" | "c++" | "cxx" => Cow::Borrowed("cpp"),
        "cs" | "csharp" => Cow::Borrowed("cs"),
        "json" => Cow::Borrowed("json"),
        "yml" | "yaml" => Cow::Borrowed("yml"),
        "md" | "markdown" => Cow::Borrowed("md"),
        "toml" => Cow::Borrowed("toml"),
        "xml" => Cow::Borrowed("xml"),
        "html" => Cow::Borrowed("html"),
        "css" => Cow::Borrowed("css"),
        "scss" => Cow::Borrowed("scss"),
        "sql" => Cow::Borrowed("sql"),
        "diff" => Cow::Borrowed("diff"),
        _ => Cow::Owned(lower),
    }
}

pub fn split_table_row(line: &str) -> Option<Vec<&str>> {
    let trimmed = line.trim();
    if !trimmed.contains('|') {
        return None;
    }

    let trimmed = trimmed
        .strip_prefix('|')
        .unwrap_or(trimmed)
        .strip_suffix('|')
        .unwrap_or(trimmed);

    // Split on pipes that are not inside backtick code spans.
    let cells = split_on_unquoted_pipe(trimmed);
    if cells.is_empty() {
        None
    } else {
        Some(cells)
    }
}

/// Split `s` on `|` characters, but skip any `|` that appears between
/// backtick (`` ` ``) pairs (inline code spans).
fn split_on_unquoted_pipe(s: &str) -> Vec<&str> {
    let bytes = s.as_bytes();
    let mut cells = Vec::new();
    let mut start = 0;
    let mut i = 0;

    while i < bytes.len() {
        if bytes[i] == b'`' {
            // Find the matching closing backtick run of the same length.
            let tick_start = i;
            while i < bytes.len() && bytes[i] == b'`' {
                i += 1;
            }
            let tick_len = i - tick_start;
            // Advance past the code span contents until we find a matching
            // closing run of backticks with the same length.
            loop {
                if i >= bytes.len() {
                    break;
                }
                if bytes[i] == b'`' {
                    let close_start = i;
                    while i < bytes.len() && bytes[i] == b'`' {
                        i += 1;
                    }
                    if i - close_start == tick_len {
                        break;
                    }
                } else {
                    i += 1;
                }
            }
        } else if bytes[i] == b'|' {
            cells.push(s[start..i].trim());
            start = i + 1;
            i += 1;
        } else {
            i += 1;
        }
    }

    cells.push(s[start..].trim());
    cells
}

pub fn parse_table_alignments(line: &str) -> Option<Vec<Option<Alignment>>> {
    let cells = split_table_row(line)?;
    let mut alignments = Vec::with_capacity(cells.len());

    for cell in cells {
        let trimmed = cell.trim();
        if trimmed.len() < 3 || !trimmed.chars().all(|ch| ch == '-' || ch == ':') {
            return None;
        }

        if trimmed.chars().filter(|&ch| ch == '-').count() < 3 {
            return None;
        }

        let alignment = match (trimmed.starts_with(':'), trimmed.ends_with(':')) {
            (true, true) => Some(Alignment::Center),
            (true, false) => Some(Alignment::Left),
            (false, true) => Some(Alignment::Right),
            (false, false) => None,
        };
        alignments.push(alignment);
    }

    Some(alignments)
}
