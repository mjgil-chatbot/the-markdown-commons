use std::collections::{HashMap, VecDeque};
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CodeBlock {
    pub language: Option<String>,
    pub code: String,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FootnoteDefinition {
    pub id: String,
    pub content: String,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LinkReferenceDefinition {
    pub label: String,
    pub url: String,
    pub title: Option<String>,
}
