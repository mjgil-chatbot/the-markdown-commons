use super::{MarkdownSegment, MarkdownTable};
use crate::common::{self, Alignment};
use crate::document::{parse_document_fragment, Block, MarkdownDocument, TableBlock};

/// Parse markdown content into a list of segments (text and tables).
///
/// This is useful when you want to handle tables differently from regular
/// text content, such as rendering tables in a custom way.
///
/// # Example
///
/// ```
/// use tmc::egui::parse_markdown_segments;
///
/// let segments = parse_markdown_segments("Hello\n\n| a | b |\n|---|---|\n\n| 1 | 2 |");
/// for segment in segments {
///     match segment {
///         tmc::egui::MarkdownSegment::Text(text) => println!("Text: {}", text),
///         tmc::egui::MarkdownSegment::Table(table) => println!("Table: {:?}", table.headers),
///     }
/// }
/// ```
pub fn parse_markdown_segments(content: &str) -> Vec<MarkdownSegment> {
    let document = parse_document_fragment(content);
    segments_from_document(&document)
}

fn segments_from_document(document: &MarkdownDocument) -> Vec<MarkdownSegment> {
    let mut segments = Vec::new();
    let mut current_text = String::new();

    for block in &document.blocks {
        match block {
            Block::Table(table) => {
                flush_text_segment(&mut segments, &mut current_text, document);
                segments.push(MarkdownSegment::Table(markdown_table_from_block(table)));
            }
            _ => {
                current_text.push_str(&block.to_markdownish());
                if !current_text.ends_with("\n\n") {
                    current_text.push('\n');
                }
            }
        }
    }

    flush_text_segment(&mut segments, &mut current_text, document);
    segments
}

fn markdown_table_from_block(table: &TableBlock) -> MarkdownTable {
    MarkdownTable {
        headers: table
            .headers
            .iter()
            .map(|cell| cell.source.clone())
            .collect(),
        alignments: table
            .alignments
            .iter()
            .map(|alignment| alignment.unwrap_or(Alignment::Left))
            .collect(),
        rows: table
            .rows
            .iter()
            .map(|row| row.iter().map(|cell| cell.source.clone()).collect())
            .collect(),
    }
}

fn flush_text_segment(
    segments: &mut Vec<MarkdownSegment>,
    current_text: &mut String,
    document: &MarkdownDocument,
) {
    if current_text.trim().is_empty() {
        current_text.clear();
        return;
    }

    if !document.references.is_empty() {
        if !current_text.ends_with('\n') {
            current_text.push('\n');
        }
        current_text.push('\n');
        current_text.push_str(&document.reference_definitions_markdown());
    }

    segments.push(MarkdownSegment::Text(std::mem::take(current_text)));
}

#[allow(dead_code)]
pub(crate) fn is_table_row(line: &str) -> bool {
    let trimmed = line.trim();
    if trimmed.is_empty() || !trimmed.contains('|') {
        return false;
    }

    if is_separator_row(trimmed) {
        return true;
    }

    trimmed.chars().filter(|&c| c == '|').count() >= 2
}

#[allow(dead_code)]
pub(crate) fn is_separator_row(line: &str) -> bool {
    let trimmed = line.trim();
    if !trimmed.contains('|') {
        return false;
    }

    let without_pipes: String = trimmed.chars().filter(|&c| c != '|').collect();

    !without_pipes.is_empty()
        && without_pipes
            .chars()
            .all(|c| c == '-' || c == ':' || c == ' ')
}

#[allow(dead_code)]
pub(crate) fn parse_table(lines: &[&str]) -> Option<MarkdownTable> {
    if lines.len() < 2 {
        return None;
    }

    let separator_idx = lines
        .iter()
        .position(|line| is_separator_row(line.trim()))?;

    if separator_idx == 0 {
        return None;
    }

    let headers = parse_table_row(lines[0]);
    if headers.is_empty() {
        return None;
    }

    let alignments = parse_alignments(lines[separator_idx], headers.len());
    let mut rows = Vec::new();

    for line in lines.iter().skip(separator_idx + 1) {
        let row = parse_table_row(line);
        if !row.is_empty() {
            let mut padded_row = row;
            while padded_row.len() < headers.len() {
                padded_row.push(String::new());
            }
            padded_row.truncate(headers.len());
            rows.push(padded_row);
        }
    }

    Some(MarkdownTable {
        headers,
        alignments,
        rows,
    })
}

#[allow(dead_code)]
pub(crate) fn parse_table_row(line: &str) -> Vec<String> {
    common::split_table_row(line)
        .unwrap_or_default()
        .into_iter()
        .map(str::to_string)
        .collect()
}

#[allow(dead_code)]
pub(crate) fn parse_alignments(line: &str, count: usize) -> Vec<Alignment> {
    common::parse_table_alignments(line)
        .unwrap_or_default()
        .into_iter()
        .map(|alignment| alignment.unwrap_or(Alignment::Left))
        .chain(std::iter::repeat(Alignment::Left))
        .take(count)
        .collect()
}
