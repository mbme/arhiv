use std::{borrow::Cow, collections::HashMap};

use anyhow::{Context, Result};
use rand::{Rng, distr::Alphanumeric};
use serde_json::Value;

#[must_use]
pub fn fuzzy_match(needle: &str, haystack: &str) -> bool {
    // if needle is empty then it matches everything
    if needle.is_empty() {
        return true;
    }

    if needle.len() > haystack.len() {
        return false;
    }

    let needle = needle.to_lowercase();
    let haystack = haystack.to_lowercase();

    if needle.len() == haystack.len() {
        return needle == haystack;
    }

    let mut haystack_chars = haystack.chars();

    'outer: for needle_char in needle.chars() {
        loop {
            if let Some(haystack_char) = haystack_chars.next() {
                if haystack_char == needle_char {
                    continue 'outer;
                }
            } else {
                return false;
            }
        }
    }

    true
}

#[must_use]
pub fn capitalize<S: Into<String>>(s: S) -> String {
    let s = s.into();

    if s.is_empty() {
        return s;
    }

    let mut iter = s.chars();
    let first_char = iter.next().unwrap().to_uppercase().to_string();

    let mut result: String = iter.collect();
    result.insert_str(0, &first_char);

    result
}

#[must_use]
pub fn generate_alpanumeric_string(length: usize) -> String {
    let mut rng = rand::rng();

    std::iter::repeat(())
        .map(|()| rng.sample(Alphanumeric))
        .map(char::from)
        .take(length)
        .collect()
}

#[must_use]
pub fn generate_alphanumeric_lines(lines: usize, line_length: usize) -> Vec<String> {
    (0..lines)
        .map(|_| generate_alpanumeric_string(line_length))
        .collect()
}

pub fn create_byte_pos_to_char_pos_map(value: &str) -> HashMap<usize, usize> {
    let mut map = HashMap::new();

    let mut byte_index = 0;
    let mut char_index = 0;
    for char in value.chars() {
        map.insert(byte_index, char_index);

        byte_index += char.len_utf8();
        char_index += 1;
    }

    map.insert(value.len(), char_index);

    map
}

pub fn generate_random_id(len: usize) -> String {
    // TODO make const fn
    let chars: Vec<char> = "0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz"
        .chars()
        .collect();

    // see https://zelark.github.io/nano-id-cc/
    nanoid::nanoid!(len, &chars)
}

pub fn value_as_string(value: Option<&Value>) -> Cow<'_, str> {
    if let Some(value) = value {
        if let Some(str) = value.as_str() {
            Cow::Borrowed(str)
        } else {
            Cow::Owned(value.to_string())
        }
    } else {
        Cow::Borrowed("")
    }
}

pub fn render_template(template: &str, value: &Value) -> Result<String> {
    let value = value.as_object().context("value must be an object")?;

    let variables = value
        .into_iter()
        .map(|(key, value)| (key.as_str(), value_as_string(Some(value))))
        .collect();

    render_template_with_vars(template, &variables)
}

pub fn render_template_with_vars<V: AsRef<str>>(
    template: &str,
    variables: &HashMap<&str, V>,
) -> Result<String> {
    subst::substitute(template, variables).context("Failed to render template")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fuzzy_match() {
        assert!(fuzzy_match("", ""));
        assert!(fuzzy_match("", "test"));
        assert!(fuzzy_match("test", "test"));
        assert!(!fuzzy_match("test", "te"));
        assert!(fuzzy_match("TEST", "teSt"));
        assert!(fuzzy_match("123", "1test2test3"));
        assert!(fuzzy_match("123", "123test2test3"));
        assert!(!fuzzy_match("123", "12test2test"));
    }

    #[test]
    fn test_capitalize() {
        assert_eq!(capitalize(""), "");
        assert_eq!(capitalize("123"), "123");
        assert_eq!(capitalize("Test"), "Test");
        assert_eq!(capitalize("test"), "Test");
    }

    #[test]
    fn test_create_byte_pos_to_char_pos_map() {
        {
            let map = create_byte_pos_to_char_pos_map("test");
            assert_eq!(*map.get(&0).unwrap(), 0);
            assert_eq!(*map.get(&1).unwrap(), 1);
            assert_eq!(*map.get(&2).unwrap(), 2);
            assert_eq!(*map.get(&3).unwrap(), 3);
            assert_eq!(*map.get(&4).unwrap(), 4);
        }

        {
            let map = create_byte_pos_to_char_pos_map("тест");
            assert_eq!(*map.get(&0).unwrap(), 0);
            assert_eq!(*map.get(&2).unwrap(), 1);
            assert_eq!(*map.get(&4).unwrap(), 2);
            assert_eq!(*map.get(&6).unwrap(), 3);
            assert_eq!(*map.get(&8).unwrap(), 4);
        }
    }
}
