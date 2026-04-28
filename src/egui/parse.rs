use crate::common;

pub(crate) fn parse_fence_start(line: &str) -> Option<(char, Option<String>)> {
    let trimmed = line.trim_start();
    let fence = common::fence_delimiter(trimmed)?;
    let fence_len = trimmed.chars().take_while(|c| *c == fence).count();
    if fence_len < 3 {
        return None;
    }
    let info = trimmed[fence_len..].trim();
    let language = info
        .split_whitespace()
        .next()
        .filter(|lang| !lang.is_empty())
        .map(|lang| common::normalize_language_tag(lang).into_owned());
    Some((fence, language))
}

pub(crate) fn parse_setext_heading(lines: &[&str], index: usize) -> Option<usize> {
    if index + 1 >= lines.len() {
        return None;
    }

    let text = lines[index].trim();
    if text.is_empty() || parse_heading(text).is_some() || parse_list_marker(lines[index]).is_some()
    {
        return None;
    }

    let underline = lines[index + 1].trim();
    if underline.len() < 3 {
        return None;
    }

    if underline.chars().all(|c| c == '=') {
        Some(1)
    } else if underline.chars().all(|c| c == '-') {
        Some(2)
    } else {
        None
    }
}

pub(crate) fn parse_heading(line: &str) -> Option<(usize, &str)> {
    let trimmed = line.trim_start();
    let level = trimmed.chars().take_while(|c| *c == '#').count();
    if !(1..=6).contains(&level) {
        return None;
    }
    let text = trimmed[level..].trim_start();
    if text.is_empty() {
        return None;
    }
    Some((level, text))
}

pub(crate) fn parse_list_marker(line: &str) -> Option<(usize, String, &str)> {
    let indent = line.chars().take_while(|c| c.is_ascii_whitespace()).count();
    let trimmed = &line[indent..];

    for marker in ["- ", "* ", "+ "] {
        if let Some(rest) = trimmed.strip_prefix(marker) {
            return Some((indent / 4, marker.trim().to_string(), rest.trim_start()));
        }
    }

    let digit_count = trimmed.chars().take_while(|c| c.is_ascii_digit()).count();
    if digit_count > 0 {
        let remainder = &trimmed[digit_count..];
        if let Some(rest) = remainder.strip_prefix(". ") {
            return Some((
                indent / 4,
                format!("{}.", &trimmed[..digit_count]),
                rest.trim_start(),
            ));
        }
    }

    None
}
