use crate::common::{self, Alignment};

pub(super) fn fence_start(line: &str) -> Option<(char, &str)> {
    let trimmed = line.trim_start();
    let marker = common::fence_delimiter(trimmed)?;
    Some((marker, trimmed.trim_start_matches(marker).trim()))
}

pub(super) fn is_indented_code_line(line: &str) -> bool {
    line.starts_with("    ") || line.starts_with('\t')
}

pub(super) fn strip_indented_code_prefix(line: &str) -> &str {
    if let Some(stripped) = line.strip_prefix("    ") {
        stripped
    } else if let Some(stripped) = line.strip_prefix('\t') {
        stripped
    } else {
        line
    }
}

pub(super) fn parse_setext_heading(lines: &[&str], index: usize) -> Option<usize> {
    if index + 1 >= lines.len() {
        return None;
    }

    let text = lines[index].trim();
    if text.is_empty() || is_special_single_line_block(lines[index]) {
        return None;
    }

    let underline = lines[index + 1].trim();
    if underline.len() < 3 {
        return None;
    }

    if underline.chars().all(|ch| ch == '=') {
        Some(1)
    } else if underline.chars().all(|ch| ch == '-') {
        Some(2)
    } else {
        None
    }
}

pub(super) fn is_fence_close(line: &str, marker: char) -> bool {
    match marker {
        '`' => line.trim_start().starts_with("```"),
        '~' => line.trim_start().starts_with("~~~"),
        _ => false,
    }
}

pub(super) fn parse_heading(line: &str) -> Option<(usize, &str, Option<&str>)> {
    let level = heading_level(line)?;
    let text = line[level..].trim_start_matches(' ').trim();
    let (heading_text, id) = split_heading_id(text);
    Some((level, heading_text, id))
}

fn split_heading_id(text: &str) -> (&str, Option<&str>) {
    if let Some(start) = text.rfind("{#") {
        if text.ends_with('}') && (start == 0 || text[..start].ends_with(' ')) {
            let id = &text[start + 2..text.len() - 1];
            if !id.is_empty() && !id.chars().any(char::is_whitespace) {
                return (text[..start].trim_end(), Some(id));
            }
        }
    }

    (text, None)
}

pub(super) fn has_hard_break(line: &str) -> bool {
    line.ends_with("  ") || line.ends_with('\\')
}

pub(super) fn trim_hard_break_marker(line: &str) -> &str {
    if let Some(stripped) = line.strip_suffix("  ") {
        stripped.trim_end()
    } else if let Some(stripped) = line.strip_suffix('\\') {
        stripped
    } else {
        line.trim()
    }
}

fn heading_level(line: &str) -> Option<usize> {
    let count = line.chars().take_while(|&ch| ch == '#').count();
    if (1..=6).contains(&count) {
        let rest = &line[count..];
        if rest.is_empty() || rest.starts_with(' ') {
            return Some(count);
        }
    }
    None
}

pub(super) fn is_horizontal_rule(line: &str) -> bool {
    let trimmed = line.trim();
    if trimmed.len() < 3 {
        return false;
    }

    let chars: Vec<char> = trimmed.chars().filter(|ch| !ch.is_whitespace()).collect();
    if chars.is_empty() {
        return false;
    }

    let first = chars[0];
    (first == '-' || first == '*' || first == '_')
        && chars.len() >= 3
        && chars.iter().all(|&ch| ch == first)
}

pub(super) fn is_unordered_list_item(line: &str) -> bool {
    let trimmed = line.trim_start();
    (trimmed.starts_with("- ") || trimmed.starts_with("* "))
        && !is_horizontal_rule(line)
        && !is_task_list_item(line)
}

pub(super) fn is_task_list_item(line: &str) -> bool {
    parse_task_list_item(line).is_some()
}

pub(super) fn parse_task_list_item(line: &str) -> Option<(bool, &str)> {
    let trimmed = line.trim_start();
    if let Some(rest) = trimmed
        .strip_prefix("- [ ] ")
        .or_else(|| trimmed.strip_prefix("* [ ] "))
    {
        return Some((false, rest));
    }

    trimmed
        .strip_prefix("- [x] ")
        .or_else(|| trimmed.strip_prefix("- [X] "))
        .or_else(|| trimmed.strip_prefix("* [x] "))
        .or_else(|| trimmed.strip_prefix("* [X] "))
        .map(|rest| (true, rest))
}

pub(super) fn is_ordered_list_item(line: &str) -> bool {
    let trimmed = line.trim_start();
    if let Some(dot_pos) = trimmed.find(". ") {
        dot_pos > 0 && trimmed[..dot_pos].chars().all(|ch| ch.is_ascii_digit())
    } else {
        false
    }
}

pub(super) fn strip_ordered_prefix(line: &str) -> &str {
    let trimmed = line.trim_start();
    if let Some(dot_pos) = trimmed.find(". ") {
        &trimmed[dot_pos + 2..]
    } else {
        trimmed
    }
}

pub(super) fn ordered_list_start(line: &str) -> Option<usize> {
    let trimmed = line.trim_start();
    let dot_pos = trimmed.find(". ")?;
    trimmed[..dot_pos].parse().ok()
}

/// A continuation line is an indented non-empty line that does not start a
/// new block-level construct (list item, heading, fence, etc.).
pub(crate) fn is_continuation_line(line: &str) -> bool {
    let trimmed = line.trim_start();
    if trimmed.is_empty() {
        return false;
    }
    let indent = line.len() - line.trim_start().len();
    if indent == 0 {
        return false;
    }
    !is_unordered_list_item(line)
        && !is_ordered_list_item(line)
        && !is_task_list_item(line)
        && !is_horizontal_rule(line)
        && heading_level(line).is_none()
        && fence_start(line).is_none()
}

pub(super) fn is_indented_sub_item(line: &str) -> bool {
    let indent = line.len() - line.trim_start().len();
    if indent == 0 {
        return false;
    }
    is_unordered_list_item(line) || is_ordered_list_item(line) || is_task_list_item(line)
}

pub(super) fn is_lazy_list_continuation_line(lines: &[&str], index: usize) -> bool {
    let line = lines[index];
    !line.trim().is_empty() && !is_special_block_start(lines, index)
}

pub(super) fn is_blank_line_followed_by_list_continuation(lines: &[&str], index: usize) -> bool {
    let next_index = index + 1;
    next_index < lines.len()
        && (is_continuation_line(lines[next_index])
            || is_indented_sub_item(lines[next_index])
            || is_lazy_list_continuation_line(lines, next_index))
}

pub(super) fn parse_table_start<'a>(
    lines: &'a [&'a str],
    index: usize,
) -> Option<(Vec<&'a str>, Vec<Option<Alignment>>)> {
    if index + 1 >= lines.len() {
        return None;
    }

    let headers = common::split_table_row(lines[index])?;
    let alignments = common::parse_table_alignments(lines[index + 1])?;
    if headers.is_empty() || headers.len() != alignments.len() {
        return None;
    }

    Some((headers, alignments))
}

pub(super) fn is_definition_list_start(lines: &[&str], index: usize) -> bool {
    if index + 1 >= lines.len() {
        return false;
    }

    let term = lines[index].trim();
    !term.is_empty()
        && !is_special_single_line_block(lines[index])
        && lines[index + 1].trim_start().starts_with(": ")
}

fn is_special_single_line_block(line: &str) -> bool {
    heading_level(line).is_some()
        || is_horizontal_rule(line)
        || is_indented_code_line(line)
        || fence_start(line).is_some()
        || line.trim_start().starts_with('>')
        || is_unordered_list_item(line)
        || is_ordered_list_item(line)
        || is_task_list_item(line)
}

pub(super) fn is_special_block_start(lines: &[&str], index: usize) -> bool {
    let line = lines[index];
    is_indented_code_line(line)
        || fence_start(line).is_some()
        || parse_setext_heading(lines, index).is_some()
        || parse_table_start(lines, index).is_some()
        || heading_level(line).is_some()
        || is_horizontal_rule(line)
        || line.trim_start().starts_with('>')
        || is_definition_list_start(lines, index)
        || is_task_list_item(line)
        || is_unordered_list_item(line)
        || is_ordered_list_item(line)
}
