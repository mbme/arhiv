use arhiv::entities::*;
pub use parser::*;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::convert::From;

mod parser;

#[derive(Serialize, Deserialize, Default)]
pub struct MarkupString(pub String);

impl MarkupString {
    pub fn parse(&self) -> Vec<Node> {
        parse_markup(&self.0)
    }
}

impl From<String> for MarkupString {
    fn from(value: String) -> Self {
        MarkupString(value)
    }
}

pub fn create_link(url: &str, text: &str) -> String {
    if text.is_empty() {
        format!("[[{}]]", url)
    } else {
        format!("[[{}][{}]]", url, text)
    }
}

pub fn extract_refs(markup: &Vec<Node>) -> HashSet<Id> {
    markup
        .iter()
        .filter_map(Node::get_children)
        .flatten()
        .filter_map(|node| {
            if let InlineNode::Link(reference, _) = node {
                Some(Id(reference.clone()))
            } else {
                None
            }
        })
        .collect()
}
