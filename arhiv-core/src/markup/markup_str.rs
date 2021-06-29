use std::collections::HashSet;
use std::convert::From;

use pulldown_cmark::{Event, Options, Parser, Tag};
use serde::{Deserialize, Serialize};

use super::utils::extract_id;
use crate::entities::*;

#[derive(Serialize, Deserialize, Debug)]
pub enum MarkupStr<'a> {
    Ref(&'a str),
    Owned(String),
}

impl<'a> MarkupStr<'a> {
    pub fn parse(&self) -> Parser {
        let mut options = Options::empty();
        options.insert(Options::ENABLE_STRIKETHROUGH);
        options.insert(Options::ENABLE_TABLES);
        options.insert(Options::ENABLE_TASKLISTS);

        Parser::new_ext(self.as_ref(), options)
    }

    pub fn extract_refs(&self) -> HashSet<Id> {
        let mut refs: HashSet<Id> = HashSet::new();

        let parser = self.parse();
        for event in parser {
            match event {
                // FIXME handle images
                Event::Start(Tag::Link(_, ref destination, _)) => {
                    if let Some(id) = extract_id(destination) {
                        refs.insert(id);
                    }
                }
                _ => {}
            }
        }

        refs
    }

    pub fn preview(&self, lines: usize) -> Self {
        self.as_ref()
            .lines()
            .take(lines)
            .collect::<Vec<_>>()
            .join("\n")
            .into()
    }
}

impl<'a> AsRef<str> for MarkupStr<'a> {
    fn as_ref(&self) -> &str {
        match self {
            MarkupStr::Ref(s) => s,
            MarkupStr::Owned(s) => &s,
        }
    }
}

impl<'a> From<&'a str> for MarkupStr<'a> {
    fn from(value: &'a str) -> Self {
        MarkupStr::Ref(value)
    }
}

impl<'a> From<String> for MarkupStr<'a> {
    fn from(value: String) -> Self {
        MarkupStr::Owned(value)
    }
}
