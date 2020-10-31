use nom::branch::alt;
use nom::bytes::complete::{is_not, tag, take, take_till1, take_while1};
use nom::combinator::{complete, map, value};
use nom::multi::{many0, many1};
use nom::sequence::{delimited, pair};

use nom::IResult;

#[derive(Clone, Debug, PartialEq)]
pub enum InlineNode {
    String(String),
    Link(String, String),
    Bold(String),
}

pub type Line = Vec<InlineNode>;

#[derive(Debug, PartialEq)]
pub enum Node {
    Newlines(usize),
    Header(String),
    Line(Line),
}

fn is_newline(c: char) -> bool {
    c == '\n'
}

// one or more newlines
pub fn parse_newlines(input: &str) -> IResult<&str, Node> {
    map(
        take_while1(is_newline), //
        |e: &str| Node::Newlines(e.len()),
    )(input)
}

pub fn parse_header(input: &str) -> IResult<&str, Node> {
    let (input, _) = tag("# ")(input)?;

    map(take_till1(is_newline), |e: &str| {
        Node::Header(e.to_string())
    })(input)
}

pub fn parse_link(input: &str) -> IResult<&str, InlineNode> {
    map(
        pair(
            delimited(tag("[["), is_not("]"), tag("]")), //
            alt((
                value("", tag("]")), //
                delimited(tag("["), is_not("]"), tag("]]")),
            )),
        ),
        |(reference, description)| InlineNode::Link(reference.to_string(), description.to_string()),
    )(input)
}

pub fn parse_bold(input: &str) -> IResult<&str, InlineNode> {
    map(
        delimited(tag("*"), is_not("*"), tag("*")), //
        |value: &str| InlineNode::Bold(value.to_string()),
    )(input)
}

fn normalize_line(items: Line) -> Line {
    let mut line: Line = Vec::new();

    let mut acc = Vec::new();

    for item in items {
        match item {
            InlineNode::String(s) => {
                acc.push(s);
            }
            node => {
                if acc.len() > 0 {
                    line.push(InlineNode::String(acc.join("")));
                    acc.clear();
                }
                line.push(node);
            }
        }
    }

    if acc.len() > 0 {
        line.push(InlineNode::String(acc.join("")));
    }

    line
}

pub fn parse_line(input: &str) -> IResult<&str, Node> {
    let (rest, input) = take_till1(is_newline)(input)?;

    let (_, result) = map(
        complete(many1(alt((
            parse_link, //
            parse_bold,
            map(take(1usize), |c: &str| InlineNode::String(c.to_string())),
        )))),
        |items| Node::Line(normalize_line(items)),
    )(input)?;

    Ok((rest, result))
}

pub fn parse_markup(input: &str) -> Vec<Node> {
    let input = input.replace("\r\n", "\n");

    let (_, result) = complete(many0(alt((
        parse_newlines, //
        parse_header,
        parse_line,
    ))))(&input)
    .expect("must be able to parse markup");

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use nom::error::ErrorKind;
    use nom::Err::Error;

    #[test]
    fn test_parse_header() {
        assert_eq!(
            parse_header("# test"),
            Ok(("", Node::Header("test".to_string())))
        );

        assert_eq!(
            parse_header("# test\nok"),
            Ok(("\nok", Node::Header("test".to_string())))
        );

        assert_eq!(parse_header("#test"), Err(Error(("#test", ErrorKind::Tag))));
    }

    #[test]
    fn test_parse_link() {
        assert_eq!(
            parse_link("[[url][description]]"),
            Ok((
                "",
                InlineNode::Link("url".to_string(), "description".to_string())
            ))
        );

        assert_eq!(
            parse_link("[[url]]"),
            Ok(("", InlineNode::Link("url".to_string(), "".to_string())))
        );

        assert_eq!(parse_link("[[]]"), Err(Error(("]]", ErrorKind::IsNot))));
    }

    #[test]
    fn test_parse_bold() {
        assert_eq!(
            parse_bold("*test *"),
            Ok(("", InlineNode::Bold("test ".to_string())))
        )
    }

    #[test]
    fn test_parse_line() {
        assert_eq!(
            parse_line("ok go*test * line\nb"),
            Ok((
                "\nb",
                Node::Line(vec![
                    InlineNode::String("ok go".to_string()),
                    InlineNode::Bold("test ".to_string()),
                    InlineNode::String(" line".to_string())
                ])
            ))
        )
    }

    #[test]
    fn test_parse_markup() {
        assert_eq!(parse_markup(""), vec![]);

        assert_eq!(
            parse_markup(
                "
# Header
line1
line2
"
            ),
            vec![
                Node::Newlines(1),
                Node::Header("Header".to_string()),
                Node::Newlines(1),
                Node::Line(vec![InlineNode::String("line1".to_string())]),
                Node::Newlines(1),
                Node::Line(vec![InlineNode::String("line2".to_string())]),
                Node::Newlines(1),
            ]
        );
    }
}
