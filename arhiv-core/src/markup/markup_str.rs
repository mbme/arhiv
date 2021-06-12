use crate::entities::*;
use pulldown_cmark::{Event, Options, Parser, Tag};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::convert::From;

use super::utils::extract_id;

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct MarkupStr<'a>(&'a str);

impl<'a> MarkupStr<'a> {
    pub(crate) fn parse(&self) -> Parser {
        let mut options = Options::empty();
        options.insert(Options::ENABLE_STRIKETHROUGH);
        options.insert(Options::ENABLE_TABLES);
        options.insert(Options::ENABLE_TASKLISTS);

        Parser::new_ext(self.0, options)
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
}

impl<'a> From<&'a str> for MarkupStr<'a> {
    fn from(value: &'a str) -> Self {
        MarkupStr(value)
    }
}
