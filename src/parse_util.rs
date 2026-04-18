pub(crate) fn starts_with_chars(chars: &[char], index: usize, pattern: &str) -> bool {
    let pattern: Vec<char> = pattern.chars().collect();
    index + pattern.len() <= chars.len()
        && chars[index..index + pattern.len()]
            .iter()
            .zip(pattern.iter())
            .all(|(left, right)| left == right)
}

pub(crate) fn find_char(chars: &[char], target: char, start: usize) -> Option<usize> {
    (start..chars.len()).find(|&index| chars[index] == target)
}

/// Find the exclusive end index of a code span starting at `start`.
/// Handles multi-backtick delimiters by matching equal-length runs.
pub(crate) fn find_code_span_end(chars: &[char], start: usize) -> Option<usize> {
    if chars.get(start) != Some(&'`') {
        return None;
    }

    // Count opening backticks
    let mut i = start;
    while i < chars.len() && chars[i] == '`' {
        i += 1;
    }
    let open_len = i - start;
    if open_len == 0 {
        return None;
    }

    // Search for closing sequence of the same length
    while i < chars.len() {
        if chars[i] == '`' {
            let mut close_len = 0;
            while i < chars.len() && chars[i] == '`' {
                close_len += 1;
                i += 1;
            }
            if close_len == open_len {
                return Some(i); // Return exclusive end (after all closing backticks)
            }
            // Different length - could be a nested span, continue searching
        } else {
            i += 1;
        }
    }

    None
}

pub(crate) fn find_sequence(chars: &[char], pattern: &str, start: usize) -> Option<usize> {
    let pattern: Vec<char> = pattern.chars().collect();
    if pattern.is_empty() || start >= chars.len() {
        return None;
    }

    (start..=chars.len().saturating_sub(pattern.len())).find(|&index| {
        chars[index..index + pattern.len()]
            .iter()
            .zip(pattern.iter())
            .all(|(left, right)| left == right)
    })
}

pub(crate) fn parse_bracket_content(chars: &[char], start: usize) -> Option<(String, usize)> {
    if chars.get(start) != Some(&'[') {
        return None;
    }

    let mut depth = 0usize;
    let mut index = start + 1;
    let mut out = String::new();

    while index < chars.len() {
        match chars[index] {
            '\\' if index + 1 < chars.len() => {
                out.push(chars[index]);
                out.push(chars[index + 1]);
                index += 2;
            }
            '[' => {
                depth += 1;
                out.push('[');
                index += 1;
            }
            ']' if depth > 0 => {
                depth -= 1;
                out.push(']');
                index += 1;
            }
            ']' => return Some((out, index + 1)),
            ch => {
                out.push(ch);
                index += 1;
            }
        }
    }

    None
}

pub(crate) fn parse_parenthesized_content(chars: &[char], start: usize) -> Option<(String, usize)> {
    if chars.get(start) != Some(&'(') {
        return None;
    }

    let mut depth = 1usize;
    let mut index = start + 1;
    let mut out = String::new();

    while index < chars.len() {
        match chars[index] {
            '\\' if index + 1 < chars.len() => {
                out.push(chars[index]);
                out.push(chars[index + 1]);
                index += 2;
            }
            '(' => {
                depth += 1;
                out.push('(');
                index += 1;
            }
            ')' => {
                depth -= 1;
                if depth == 0 {
                    return Some((out, index + 1));
                }
                out.push(')');
                index += 1;
            }
            ch => {
                out.push(ch);
                index += 1;
            }
        }
    }

    None
}

pub(crate) fn parse_destination_and_title(spec: &str) -> Option<(String, Option<String>)> {
    let spec = spec.trim();
    if spec.is_empty() {
        return None;
    }

    let (url, rest) = if let Some(stripped) = spec.strip_prefix('<') {
        let end = stripped.find('>')?;
        (stripped[..end].to_string(), stripped[end + 1..].trim())
    } else if let Some(space) = spec.find(char::is_whitespace) {
        (spec[..space].to_string(), spec[space..].trim())
    } else {
        (spec.to_string(), "")
    };

    let title = parse_optional_title(rest);
    Some((url, title))
}

pub(crate) fn parse_optional_title(spec: &str) -> Option<String> {
    let spec = spec.trim();
    if spec.is_empty() {
        return None;
    }

    if spec.len() >= 2 {
        let bytes = spec.as_bytes();
        let first = bytes[0] as char;
        let last = bytes[spec.len() - 1] as char;
        let wraps = matches!((first, last), ('"', '"') | ('\'', '\'') | ('(', ')'));
        if wraps {
            return Some(spec[1..spec.len() - 1].to_string());
        }
    }

    None
}

pub(crate) fn normalize_reference_label(label: &str) -> String {
    label.trim().to_ascii_lowercase()
}

pub(crate) fn parse_link_destination(
    chars: &[char],
    start: usize,
) -> Option<(String, Option<String>, usize)> {
    let (content, end) = parse_parenthesized_content(chars, start)?;
    let (url, title) = parse_destination_and_title(&content)?;
    Some((url, title, end))
}

pub(crate) fn parse_footnote_reference(chars: &[char], start: usize) -> Option<(String, usize)> {
    if chars.get(start) != Some(&'[') || chars.get(start + 1) != Some(&'^') {
        return None;
    }

    let end = find_char(chars, ']', start + 2)?;
    let id: String = chars[start + 2..end].iter().collect();
    if id.is_empty() || id.chars().any(char::is_whitespace) {
        return None;
    }

    Some((id, end + 1))
}

pub(crate) fn parse_autolink(chars: &[char], start: usize) -> (String, usize, usize) {
    let mut end = start;
    while end < chars.len() && !chars[end].is_whitespace() && chars[end] != '<' {
        end += 1;
    }

    let mut trimmed_end = end;
    while trimmed_end > start {
        let ch = chars[trimmed_end - 1];
        if ch == ')' || ch == ']' {
            // Only trim if the bracket is unbalanced (no matching opening bracket)
            if !has_matching_opening(chars, start, trimmed_end - 1) {
                trimmed_end -= 1;
            } else {
                break;
            }
        } else if matches!(ch, '.' | ',' | '!' | '?' | ':' | ';') {
            trimmed_end -= 1;
        } else {
            break;
        }
    }

    let url: String = chars[start..trimmed_end].iter().collect();
    (url, trimmed_end, end)
}

fn has_matching_opening(chars: &[char], url_start: usize, close_pos: usize) -> bool {
    let close_char = chars[close_pos];
    let open_char = if close_char == ')' { '(' } else { '[' };

    // Count opening brackets in the URL that don't have a matching closing bracket
    let mut open_count = 0;
    for i in url_start..close_pos {
        if chars[i] == open_char {
            open_count += 1;
        } else if chars[i] == close_char {
            if open_count > 0 {
                open_count -= 1;
            } else {
                // Unmatched closing bracket - this is unbalanced
                return false;
            }
        }
    }
    // If there are unmatched opening brackets, the closing bracket is balanced
    open_count > 0
}

pub(crate) fn parse_angle_bracket_content(chars: &[char], start: usize) -> Option<(String, usize)> {
    if chars.get(start) != Some(&'<') {
        return None;
    }

    let end = find_char(chars, '>', start + 1)?;
    Some((chars[start + 1..end].iter().collect(), end + 1))
}

pub(crate) fn parse_html_entity(chars: &[char], start: usize) -> Option<(String, usize)> {
    if chars.get(start) != Some(&'&') {
        return None;
    }

    let end = find_char(chars, ';', start + 1)?;
    let entity: String = chars[start..=end].iter().collect();
    let body = &entity[1..entity.len() - 1];
    let valid = body.starts_with('#') || body.chars().all(|ch| ch.is_ascii_alphanumeric());
    if valid {
        Some((entity, end + 1))
    } else {
        None
    }
}

pub(crate) fn looks_like_url(value: &str) -> bool {
    value.starts_with("http://") || value.starts_with("https://")
}

pub(crate) fn looks_like_email(value: &str) -> bool {
    value.contains('@') && !value.contains(char::is_whitespace) && !value.contains('/')
}

pub(crate) fn looks_like_html_tag(value: &str) -> bool {
    let trimmed = value.trim();
    if trimmed.is_empty() || trimmed.contains('@') || looks_like_url(trimmed) {
        return false;
    }

    trimmed.starts_with('/')
        || trimmed.starts_with('!')
        || trimmed
            .chars()
            .next()
            .map(|ch| ch.is_ascii_alphabetic())
            .unwrap_or(false)
}

pub(crate) fn is_markdown_escapable(ch: char) -> bool {
    matches!(
        ch,
        '\\' | '`'
            | '*'
            | '_'
            | '{'
            | '}'
            | '['
            | ']'
            | '<'
            | '>'
            | '('
            | ')'
            | '#'
            | '+'
            | '-'
            | '.'
            | '!'
            | '|'
    )
}

pub(crate) fn parse_emoji_shortcode(chars: &[char], start: usize) -> Option<(&'static str, usize)> {
    if chars[start] != ':' {
        return None;
    }

    let end = find_char(chars, ':', start + 1)?;
    if end == start + 1 {
        return None;
    }

    let code: String = chars[start + 1..end].iter().collect();
    if !code.chars().all(|ch| {
        ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '_' || ch == '+' || ch == '-'
    }) {
        return None;
    }

    let emoji = match code.as_str() {
        "rocket" => "🚀",
        "sparkles" => "✨",
        "joy" => "😂",
        "tent" => "⛺",
        "white_check_mark" => "✅",
        "memo" => "📝",
        "fire" => "🔥",
        "wave" => "👋",
        _ => return None,
    };

    Some((emoji, end + 1))
}
