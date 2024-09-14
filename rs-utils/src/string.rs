use std::collections::HashMap;

use anyhow::{Context, Result};
use data_encoding::{BASE64, BASE64URL, HEXUPPER};

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
    use rand::distributions::Alphanumeric;
    use rand::prelude::*;

    let mut rng = thread_rng();

    std::iter::repeat(())
        .map(|()| rng.sample(Alphanumeric))
        .map(char::from)
        .take(length)
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

pub fn generate_random_id() -> String {
    // TODO make const fn
    let chars: Vec<char> = "0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz"
        .chars()
        .collect();

    // see https://zelark.github.io/nano-id-cc/
    nanoid::nanoid!(14, &chars)
}

#[must_use]
pub fn to_url_safe_base64(bytes: &[u8]) -> String {
    BASE64URL.encode(bytes)
}

#[must_use]
pub fn is_valid_base64(value: &str) -> bool {
    BASE64URL.decode(value.as_bytes()).is_ok()
}

pub fn decode_base64(data: &str) -> Result<Vec<u8>> {
    BASE64
        .decode(data.as_bytes())
        .context("Failed to decode base64 string")
}

pub fn bytes_to_hex_string(bytes: &[u8]) -> String {
    HEXUPPER.encode(bytes)
}

pub fn hex_string_to_bytes(hex: &str) -> Result<Vec<u8>> {
    HEXUPPER
        .decode(hex.as_bytes())
        .context("Failed to decode hex string")
}

#[cfg(test)]
mod tests {
    use rand::RngCore;

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

    #[test]
    fn test_hex_encode_decode() {
        let mut data = [0u8; 150];
        rand::thread_rng().fill_bytes(&mut data);

        let result = bytes_to_hex_string(&data);
        let result = hex_string_to_bytes(&result).unwrap();

        assert_eq!(data, result.as_slice());
    }
}
