use std::collections::HashSet;
use std::convert::From;

use pulldown_cmark::{CowStr, Event, Options, Parser, Tag};

use super::utils::extract_id;
use crate::entities::*;

pub struct MarkupStr<'a>(CowStr<'a>);

impl<'a> MarkupStr<'a> {
    #[must_use]
    pub fn parse(&self) -> Parser {
        let mut options = Options::empty();
        options.insert(Options::ENABLE_STRIKETHROUGH);
        options.insert(Options::ENABLE_TABLES);
        options.insert(Options::ENABLE_TASKLISTS);

        Parser::new_ext(self.0.as_ref(), options)
    }

    #[must_use]
    pub fn extract_refs(&self) -> HashSet<Id> {
        let mut refs: HashSet<Id> = HashSet::new();

        let parser = self.parse();
        for event in parser {
            if let Event::Start(Tag::Link(_, ref destination, _)) = event {
                if let Some(id) = extract_id(destination) {
                    refs.insert(id);
                }
            }
        }

        refs
    }

    #[must_use]
    pub fn preview(&self, lines: usize) -> Self {
        self.0
            .as_ref()
            .lines()
            .take(lines)
            .collect::<Vec<_>>()
            .join("\n")
            .into()
    }
}

impl<'a> From<&'a str> for MarkupStr<'a> {
    fn from(value: &'a str) -> Self {
        MarkupStr(CowStr::Borrowed(value))
    }
}

impl<'a> From<String> for MarkupStr<'a> {
    fn from(value: String) -> Self {
        MarkupStr(CowStr::Boxed(value.into_boxed_str()))
    }
}
