use super::{Alignment, Block, MarkdownDocument, ParagraphBlock, ParagraphLine, TableBlock};

impl MarkdownDocument {
    pub fn reference_definitions_markdown(&self) -> String {
        let mut output = String::new();

        for reference in &self.references {
            output.push('[');
            output.push_str(&reference.label);
            output.push_str("]: ");
            output.push_str(&reference.url);
            if let Some(title) = &reference.title {
                output.push(' ');
                output.push('"');
                output.push_str(title);
                output.push('"');
            }
            output.push('\n');
        }

        output
    }
}

impl Block {
    pub fn to_markdownish(&self) -> String {
        match self {
            Self::Paragraph(paragraph) => paragraph.to_markdownish(),
            Self::Heading(heading) => {
                let mut line = format!("{} {}", "#".repeat(heading.level), heading.content.source);
                if let Some(id) = &heading.id {
                    line.push_str(" {#");
                    line.push_str(id);
                    line.push('}');
                }
                line.push('\n');
                line
            }
            Self::CodeBlock(code) => {
                let mut output = String::from("```");
                if let Some(language) = &code.language {
                    output.push_str(language);
                }
                output.push('\n');
                output.push_str(&code.code);
                if !code.code.ends_with('\n') {
                    output.push('\n');
                }
                output.push_str("```\n");
                output
            }
            Self::Table(table) => table.to_markdownish(),
            Self::BlockQuote(blocks) => {
                let mut output = String::new();
                for block in blocks {
                    for line in block.to_markdownish().lines() {
                        output.push_str("> ");
                        output.push_str(line);
                        output.push('\n');
                    }
                }
                output
            }
            Self::DefinitionList(items) => {
                let mut output = String::new();
                for item in items {
                    output.push_str(&item.term.source);
                    output.push('\n');
                    for definition in &item.definitions {
                        output.push_str(": ");
                        output.push_str(&definition.source);
                        output.push('\n');
                    }
                }
                output
            }
            Self::TaskList(items) => {
                let mut output = String::new();
                for item in items {
                    output.push_str(if item.checked { "- [x] " } else { "- [ ] " });
                    output.push_str(&item.content.source);
                    output.push('\n');
                }
                output
            }
            Self::UnorderedList(list) => {
                let mut output = String::new();
                let indent = " ".repeat(list.indent);
                for item in &list.items {
                    output.push_str(&indent);
                    output.push_str("- ");
                    output.push_str(&item.content.source);
                    output.push('\n');
                }
                output
            }
            Self::OrderedList(list) => {
                let mut output = String::new();
                let indent = " ".repeat(list.indent);
                for (index, item) in list.items.iter().enumerate() {
                    output.push_str(&indent);
                    output.push_str(&(list.start + index).to_string());
                    output.push_str(". ");
                    output.push_str(&item.content.source);
                    output.push('\n');
                }
                output
            }
            Self::HorizontalRule => String::from("---\n"),
        }
    }
}

impl ParagraphBlock {
    pub fn to_markdownish(&self) -> String {
        let mut output = paragraph_source_from_lines(&self.lines);
        output.push('\n');
        output
    }
}

pub(super) fn paragraph_source_from_lines(lines: &[ParagraphLine]) -> String {
    let mut output = String::new();
    for (index, line) in lines.iter().enumerate() {
        if index > 0 {
            output.push('\n');
        }
        output.push_str(&line.content.source);
        if line.hard_break {
            output.push_str("  ");
        }
    }
    output
}

impl TableBlock {
    pub fn to_markdownish(&self) -> String {
        let mut output = String::new();

        output.push('|');
        for header in &self.headers {
            output.push(' ');
            output.push_str(&header.source);
            output.push(' ');
            output.push('|');
        }
        output.push('\n');

        output.push('|');
        for alignment in &self.alignments {
            let cell = match alignment {
                Some(Alignment::Left) => ":---",
                Some(Alignment::Center) => ":---:",
                Some(Alignment::Right) => "---:",
                None => "---",
            };
            output.push(' ');
            output.push_str(cell);
            output.push(' ');
            output.push('|');
        }
        output.push('\n');

        for row in &self.rows {
            output.push('|');
            for cell in row {
                output.push(' ');
                output.push_str(&cell.source);
                output.push(' ');
                output.push('|');
            }
            output.push('\n');
        }

        output
    }
}
