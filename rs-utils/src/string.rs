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
}
