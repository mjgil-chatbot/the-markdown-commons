use crate::common;
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
