use std::convert::From;
use std::{collections::HashSet, ops::Range};

use anyhow::{bail, ensure, Context, Result};
use pulldown_cmark::{
    Alignment, CodeBlockKind, CowStr, Event, HeadingLevel, LinkType, Options, Parser, Tag,
};
use serde::Serialize;

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

    pub fn get_ast(&self) -> Result<MarkupElement<'_>> {
        let mut stack: Vec<MarkupElement<'_>> = vec![MarkupElement::Document { children: vec![] }];

        for (event, range) in self.parse().into_offset_iter() {
            match event {
                Event::Text(value) => {
                    stack
                        .last_mut()
                        .context("stack must not be empty")?
                        .add_child(MarkupElement::Text { value, range })?;
                }
                Event::Code(value) => {
                    stack
                        .last_mut()
                        .context("stack must not be empty")?
                        .add_child(MarkupElement::Code { value, range })?;
                }
                Event::Html(value) => {
                    stack
                        .last_mut()
                        .context("stack must not be empty")?
                        .add_child(MarkupElement::Html { value, range })?;
                }
                Event::FootnoteReference(label) => {
                    stack
                        .last_mut()
                        .context("stack must not be empty")?
                        .add_child(MarkupElement::FootnoteReference { label, range })?;
                }
                Event::SoftBreak => {
                    stack
                        .last_mut()
                        .context("stack must not be empty")?
                        .add_child(MarkupElement::SoftBreak { range })?;
                }
                Event::HardBreak => {
                    stack
                        .last_mut()
                        .context("stack must not be empty")?
                        .add_child(MarkupElement::HardBreak { range })?;
                }
                Event::Rule => {
                    stack
                        .last_mut()
                        .context("stack must not be empty")?
                        .add_child(MarkupElement::Rule { range })?;
                }
                Event::TaskListMarker(checked) => {
                    stack
                        .last_mut()
                        .context("stack must not be empty")?
                        .add_child(MarkupElement::TaskListMarker { checked, range })?;
                }

                Event::Start(tag) => {
                    let el = match tag {
                        Tag::Paragraph => MarkupElement::Paragraph {
                            children: vec![],
                            range,
                        },
                        Tag::Heading(level, ..) => MarkupElement::Heading {
                            range,
                            level,
                            children: vec![],
                        },
                        Tag::BlockQuote => MarkupElement::BlockQuote {
                            children: vec![],
                            range,
                        },
                        Tag::CodeBlock(kind) => MarkupElement::CodeBlock {
                            range,
                            kind,
                            children: vec![],
                        },
                        Tag::List(first_item_number) => MarkupElement::List {
                            range,
                            first_item_number,
                            children: vec![],
                        },
                        Tag::Item => MarkupElement::ListItem {
                            children: vec![],
                            range,
                        },
                        Tag::FootnoteDefinition(label) => MarkupElement::FootnoteDefinition {
                            range,
                            label,
                            children: vec![],
                        },
                        Tag::Table(alignments) => MarkupElement::Table {
                            range,
                            alignments,
                            children: vec![],
                        },
                        Tag::TableHead => MarkupElement::TableHead {
                            children: vec![],
                            range,
                        },
                        Tag::TableRow => MarkupElement::TableRow {
                            children: vec![],
                            range,
                        },
                        Tag::TableCell => MarkupElement::TableCell {
                            children: vec![],
                            range,
                        },

                        Tag::Emphasis => MarkupElement::Emphasis {
                            children: vec![],
                            range,
                        },
                        Tag::Strong => MarkupElement::Strong {
                            children: vec![],
                            range,
                        },
                        Tag::Strikethrough => MarkupElement::Strikethrough {
                            children: vec![],
                            range,
                        },

                        Tag::Link(link_type, url, title) => MarkupElement::Link {
                            link_type,
                            url,
                            title,
                            children: vec![],
                            range,
                        },

                        Tag::Image(link_type, url, title) => MarkupElement::Image {
                            link_type,
                            url,
                            title,
                            children: vec![],
                            range,
                        },
                    };

                    stack.push(el);
                }

                Event::End(_tag) => {
                    let el = stack.pop().context("stack must not be empty")?;
                    stack
                        .last_mut()
                        .context("stack must not be empty")?
                        .add_child(el)?;
                }
            }
        }

        ensure!(stack.len() == 1, "only Document must be on stack");

        stack.pop().context("Document must be on stack")
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

pub type Children<'a> = Vec<MarkupElement<'a>>;

#[derive(Serialize, Debug)]
#[serde(deny_unknown_fields, tag = "typeName")]
pub enum MarkupElement<'a> {
    Document {
        children: Children<'a>,
    },
    Text {
        range: Range<usize>,
        value: CowStr<'a>,
    },
    Code {
        range: Range<usize>,
        value: CowStr<'a>,
    },
    Html {
        range: Range<usize>,
        value: CowStr<'a>,
    },
    FootnoteReference {
        range: Range<usize>,
        label: CowStr<'a>,
    },
    SoftBreak {
        range: Range<usize>,
    },
    HardBreak {
        range: Range<usize>,
    },
    Rule {
        range: Range<usize>,
    },
    TaskListMarker {
        range: Range<usize>,
        checked: bool,
    },
    Paragraph {
        range: Range<usize>,
        children: Children<'a>,
    },
    Heading {
        range: Range<usize>,
        level: HeadingLevel,
        children: Children<'a>,
    },
    BlockQuote {
        range: Range<usize>,
        children: Children<'a>,
    },
    CodeBlock {
        range: Range<usize>,
        kind: CodeBlockKind<'a>,
        children: Children<'a>,
    },
    List {
        range: Range<usize>,
        first_item_number: Option<u64>,
        children: Children<'a>,
    },
    ListItem {
        range: Range<usize>,
        children: Children<'a>,
    },
    FootnoteDefinition {
        range: Range<usize>,
        label: CowStr<'a>,
        children: Children<'a>,
    },
    Table {
        range: Range<usize>,
        alignments: Vec<Alignment>,
        children: Children<'a>,
    },
    TableHead {
        range: Range<usize>,
        children: Children<'a>, // only TableCell
    },
    TableRow {
        range: Range<usize>,
        children: Children<'a>, // only TableCell
    },
    TableCell {
        range: Range<usize>,
        children: Children<'a>,
    },

    Emphasis {
        range: Range<usize>,
        children: Children<'a>,
    },
    Strong {
        range: Range<usize>,
        children: Children<'a>,
    },
    Strikethrough {
        range: Range<usize>,
        children: Children<'a>,
    },

    Link {
        range: Range<usize>,
        link_type: LinkType,
        url: CowStr<'a>,
        title: CowStr<'a>,
        children: Children<'a>,
    },

    Image {
        range: Range<usize>,
        link_type: LinkType,
        url: CowStr<'a>,
        title: CowStr<'a>,
        children: Children<'a>,
    },
}

impl<'a> MarkupElement<'a> {
    fn get_children_mut(&mut self) -> Option<&mut Children<'a>> {
        match self {
            MarkupElement::Document { children }
            | MarkupElement::Paragraph { children, .. }
            | MarkupElement::Heading { children, .. }
            | MarkupElement::BlockQuote { children, .. }
            | MarkupElement::CodeBlock { children, .. }
            | MarkupElement::List { children, .. }
            | MarkupElement::ListItem { children, .. }
            | MarkupElement::FootnoteDefinition { children, .. }
            | MarkupElement::Table { children, .. }
            | MarkupElement::TableHead { children, .. }
            | MarkupElement::TableRow { children, .. }
            | MarkupElement::TableCell { children, .. }
            | MarkupElement::Emphasis { children, .. }
            | MarkupElement::Strong { children, .. }
            | MarkupElement::Strikethrough { children, .. }
            | MarkupElement::Link { children, .. }
            | MarkupElement::Image { children, .. } => {
                //
                Some(children)
            }

            _ => None,
        }
    }

    fn add_child(&mut self, child: MarkupElement<'a>) -> Result<()> {
        if let Some(children) = self.get_children_mut() {
            children.push(child);

            Ok(())
        } else {
            bail!("can't push child into {:?}", self)
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::Value;

    use crate::markup::MarkupStr;

    fn into_ast(value: &str) -> Value {
        let markup = MarkupStr::from(value);

        let ast = markup.get_ast().expect("must get ast");

        serde_json::to_value(ast).expect("ast must serialize")
    }

    #[test]
    fn test_parse_header() {
        let ast = into_ast("# TEST HEADER");

        insta::assert_json_snapshot!(ast);
    }

    #[test]
    fn test_parse_list() {
        let ast = into_ast(
            "
- [ ] test 1
- [+] test 2
- [x] test 3
- test 4
* test 5
",
        );

        insta::assert_json_snapshot!(ast);
    }

    #[test]
    fn test_parse_hr() {
        let ast = into_ast("---");

        insta::assert_json_snapshot!(ast);
    }

    #[test]
    fn test_parse_table() {
        let ast = into_ast(
            "
| Syntax      | Description | Test Text     |
| :---        |    :----:   |          ---: |
| Header      | Title       | Here's this   |
| Paragraph   | Text        | And more      |
",
        );

        insta::assert_json_snapshot!(ast);
    }

    #[test]
    fn test_parse_code_block() {
        let ast = into_ast(
            r#"
```json
{
  "firstName": "John",
  "lastName": "Smith",
  "age": 25
}
```
"#,
        );

        insta::assert_json_snapshot!(ast);
    }

    #[test]
    fn test_parse_link() {
        let ast = into_ast(r#"[test](http://example.com "some title")"#);

        insta::assert_json_snapshot!(ast);
    }

    #[test]
    fn test_parse_autolink() {
        let ast = into_ast(r#"<http://example.com>"#);

        insta::assert_json_snapshot!(ast);
    }

    #[test]
    fn test_parse_image() {
        let ast = into_ast(r#"![test](http://example.com "some title")"#);

        insta::assert_json_snapshot!(ast);
    }
}
