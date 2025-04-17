use core::fmt;
use std::time::Duration;

use similar::{Algorithm, ChangeTag, TextDiff};

const DEFAULT_MERGE_TIMEOUT: Duration = Duration::from_secs(10);

#[derive(Clone)]
enum Change<'s> {
    Equal { value: &'s str },
    Delete { value: &'s str },
    Insert { value: &'s str },
}

impl fmt::Display for Change<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self {
            Change::Equal { value } => write!(f, "={value}"),
            Change::Delete { value } => write!(f, "-{value}"),
            Change::Insert { value } => write!(f, "+{value}"),
        }
    }
}
impl fmt::Debug for Change<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}", &self)
    }
}

fn get_word_diff<'s>(a: &'s str, b: &'s str) -> Vec<Change<'s>> {
    let diff = TextDiff::configure()
        .algorithm(Algorithm::Patience)
        .timeout(DEFAULT_MERGE_TIMEOUT)
        .diff_unicode_words(a, b);

    let all_changes = diff
        .iter_all_changes()
        .map(|change| {
            let value = *change.value_ref();

            match change.tag() {
                ChangeTag::Equal => Change::Equal { value },
                ChangeTag::Delete => Change::Delete { value },
                ChangeTag::Insert => Change::Insert { value },
            }
        })
        .collect::<Vec<_>>();

    all_changes
}

// TODO case insensitivity? (prefer capital letters?);
fn merge_strings(a: &str, b: &str) -> String {
    let all_changes = get_word_diff(a, b);

    let mut merged = String::new();

    // check if strings have non-whitespace equal changes
    let strings_have_common_chunks = all_changes.iter().any(|change| {
        if let Change::Equal { value } = change {
            let is_empty = value.chars().all(|c| c.is_whitespace());

            !is_empty
        } else {
            false
        }
    });

    if !strings_have_common_chunks {
        merged.push_str(a);

        if should_add_whitespace(a, b) {
            merged.push(' ');
        }

        merged.push_str(b);

        return merged;
    }

    let mut iter = all_changes.into_iter().peekable();

    while let Some(change) = iter.next() {
        match change {
            Change::Equal { value } => {
                merged.push_str(value);
            }

            Change::Delete { value } => {
                merged.push_str(value);

                if let Some(Change::Insert { value: next_value }) = iter.peek() {
                    // insert after delete means replace
                    if value.to_lowercase() == next_value.to_lowercase() {
                        iter.next();
                        continue;
                    }

                    // word A was replaced by word B
                    // We're going to insert both, so let's add a separator between them if necessary
                    if should_add_whitespace(value, next_value) {
                        merged.push(' ');
                    }
                }
            }

            Change::Insert { value } => {
                merged.push_str(value);
            }
        }
    }

    merged
}

fn should_add_whitespace(left: &str, right: &str) -> bool {
    let left_has_whitespace = left.chars().next_back().is_none_or(|c| c.is_whitespace());

    if left_has_whitespace {
        return false;
    }

    let right_has_whitespace = right.chars().next().is_none_or(|c| c.is_whitespace());

    if right_has_whitespace {
        return false;
    }

    true
}

pub fn merge_strings_three_way(base_text: &str, left_text: &str, right_text: &str) -> String {
    let left_diff = get_word_diff(base_text, left_text);
    let mut left_iter = left_diff.into_iter().peekable();

    let right_diff = get_word_diff(base_text, right_text);
    let mut right_iter = right_diff.into_iter().peekable();

    let mut merged = String::new();

    let mut left = left_iter.next();
    let mut right = right_iter.next();

    loop {
        match (left.clone(), right.clone()) {
            (None, None) => break,
            (None, Some(right_change)) => {
                match right_change {
                    Change::Equal { value } | Change::Insert { value } => {
                        merged.push_str(value);
                    }
                    Change::Delete { value: _ } => {}
                }

                right = right_iter.next();
            }
            (Some(left_change), None) => {
                match left_change {
                    Change::Equal { value } | Change::Insert { value } => {
                        merged.push_str(value);
                    }
                    Change::Delete { value: _ } => {}
                }

                left = left_iter.next();
            }
            (Some(left_change), Some(right_change)) => {
                match (left_change, right_change) {
                    (Change::Equal { value: left_value }, Change::Equal { value: _ }) => {
                        merged.push_str(left_value);

                        left = left_iter.next();
                        right = right_iter.next();
                    }

                    // one of the sides inserted
                    (Change::Equal { value: _ }, Change::Insert { value: right_value })
                    | (Change::Delete { value: _ }, Change::Insert { value: right_value }) => {
                        merged.push_str(right_value);

                        right = right_iter.next();
                    }
                    (Change::Insert { value: left_value }, Change::Equal { value: _ })
                    | (Change::Insert { value: left_value }, Change::Delete { value: _ }) => {
                        merged.push_str(left_value);

                        left = left_iter.next();
                    }

                    // one or both deleted
                    (Change::Equal { value: _ }, Change::Delete { value: _ })
                    | (Change::Delete { value: _ }, Change::Equal { value: _ })
                    | (Change::Delete { value: _ }, Change::Delete { value: _ }) => {
                        left = left_iter.next();
                        right = right_iter.next();
                    }

                    (
                        Change::Insert { value: left_value },
                        Change::Insert { value: right_value },
                    ) => {
                        // collect all sequential inserts in the left text
                        let mut left_insert = vec![left_value];
                        loop {
                            left = left_iter.next();

                            match left {
                                Some(Change::Insert { value: left_value }) => {
                                    left_insert.push(left_value);
                                }
                                _ => {
                                    break;
                                }
                            }
                        }
                        let left_insert = left_insert.join("");

                        // collect all sequential inserts in the right text
                        let mut right_insert = vec![right_value];
                        loop {
                            right = right_iter.next();

                            match right {
                                Some(Change::Insert { value: right_value }) => {
                                    right_insert.push(right_value);
                                }
                                _ => {
                                    break;
                                }
                            }
                        }
                        let right_insert = right_insert.join("");

                        // merge sequential inserts
                        merged.push_str(&merge_strings(&left_insert, &right_insert));
                    }
                }
            }
        }
    }

    merged
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_merge_strings() {
        assert_eq!(
            merge_strings("a good test", "a bad test"),
            "a good bad test"
        );

        assert_eq!(
            merge_strings("a good test", "a  bad test"),
            "a good  bad test"
        );

        assert_eq!(
            merge_strings("a good test says something", "a bad test"),
            "a good bad test says something"
        );

        // first all left, then all right
        assert_eq!(
            merge_strings("test 123", "ok go other"),
            "test 123 ok go other"
        );

        // merge common prefix
        assert_eq!(
            merge_strings("test 123", "test ok go other"),
            "test 123 ok go other"
        );

        // word boundaries
        assert_eq!(
            merge_strings(
                "A good test. Another sentence? no",
                "A good text. Another word"
            ),
            "A good test text. Another sentence? no word"
        );
    }

    #[test]
    fn test_merge_strings_three_way() {
        // no changes
        assert_eq!(
            merge_strings_three_way(
                "The quick brown fox",
                "The quick brown fox",
                "The quick brown fox"
            ),
            "The quick brown fox"
        );

        // non-conflicting insertions
        assert_eq!(
            merge_strings_three_way("a good test", "a bad test", "a good text"),
            "a bad text"
        );

        // conflicting insertions
        assert_eq!(
            merge_strings_three_way("a good test", "a bad test", "a cool test"),
            "a bad cool test"
        );

        // conflicting insertions with the same input
        assert_eq!(
            merge_strings_three_way("a good test", "a bad text", "a good text"),
            "a bad text"
        );

        // deletion in one branch
        assert_eq!(
            merge_strings_three_way(
                "The quick brown fox jumps",
                "The quick brown fox jumps",
                "The quick fox jumps",
            ),
            "The quick fox jumps"
        );

        // conflicting deletion and insertion
        assert_eq!(
            merge_strings_three_way(
                "The quick brown fox",
                "The quick fox",
                "The quick red brown fox"
            ),
            "The quick red fox"
        );

        // completely divergent
        assert_eq!(
            merge_strings_three_way("Hello world", "Hello universe", "Greetings world"),
            "Greetings universe"
        );

        // empty base
        assert_eq!(
            merge_strings_three_way("", "Hello universe", "Greetings world"),
            "Hello universe Greetings world"
        );

        // empty base, similar edits
        assert_eq!(
            merge_strings_three_way("", "Hello universe", "Hello universe and more"),
            "Hello universe and more"
        );
    }
}
