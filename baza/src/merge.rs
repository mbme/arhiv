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

impl<'s> From<similar::Change<&'s str>> for Change<'s> {
    fn from(change: similar::Change<&'s str>) -> Self {
        let value = *change.value_ref();

        match change.tag() {
            ChangeTag::Equal => Change::Equal { value },
            ChangeTag::Delete => Change::Delete { value },
            ChangeTag::Insert => Change::Insert { value },
        }
    }
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

trait Merger<'s> {
    fn merge_two_values(&mut self, left: &'s str, right: &'s str);

    fn push(&mut self, value: &'s str);

    fn push_all(&mut self, values: &[&'s str]) {
        for value in values {
            self.push(value);
        }
    }

    fn merge_diffs(&mut self, left_diff: &[Change<'s>], right_diff: &[Change<'s>]) {
        let mut left_iter = left_diff.iter().peekable();
        let mut right_iter = right_diff.iter().peekable();

        let mut left = left_iter.next();
        let mut right = right_iter.next();

        loop {
            match (left, right) {
                (None, None) => break,
                (None, Some(right_change)) => {
                    match right_change {
                        Change::Equal { value } | Change::Insert { value } => {
                            self.push(value);
                        }
                        Change::Delete { value: _ } => {}
                    }

                    right = right_iter.next();
                }
                (Some(left_change), None) => {
                    match left_change {
                        Change::Equal { value } | Change::Insert { value } => {
                            self.push(value);
                        }
                        Change::Delete { value: _ } => {}
                    }

                    left = left_iter.next();
                }
                (Some(left_change), Some(right_change)) => {
                    match (left_change, right_change) {
                        (Change::Equal { value: left_value }, Change::Equal { value: _ }) => {
                            self.push(left_value);

                            left = left_iter.next();
                            right = right_iter.next();
                        }

                        // one of the sides inserted
                        (Change::Equal { value: _ }, Change::Insert { value: right_value })
                        | (Change::Delete { value: _ }, Change::Insert { value: right_value }) => {
                            self.push(right_value);

                            right = right_iter.next();
                        }
                        (Change::Insert { value: left_value }, Change::Equal { value: _ })
                        | (Change::Insert { value: left_value }, Change::Delete { value: _ }) => {
                            self.push(left_value);

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
                            // collect all sequential inserts in the left sequence
                            let mut left_inserts = vec![*left_value];
                            loop {
                                left = left_iter.next();

                                match left {
                                    Some(Change::Insert { value: left_value }) => {
                                        left_inserts.push(*left_value);
                                    }
                                    _ => {
                                        break;
                                    }
                                }
                            }

                            // collect all sequential inserts in the right sequence
                            let mut right_inserts = vec![*right_value];
                            loop {
                                right = right_iter.next();

                                match right {
                                    Some(Change::Insert { value: right_value }) => {
                                        right_inserts.push(*right_value);
                                    }
                                    _ => {
                                        break;
                                    }
                                }
                            }

                            self.merge_inserts(&left_inserts, &right_inserts);
                        }
                    }
                }
            }
        }
    }

    fn merge_inserts(&mut self, left: &[&'s str], right: &[&'s str]) {
        let all_changes = get_slice_diff(left, right);

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
            let left_len = left.len();

            self.push_all(&left[..left_len - 1]);

            self.merge_two_values(left[left_len - 1], right[0]);

            self.push_all(&right[1..]);

            return;
        }

        let mut iter = all_changes.into_iter().peekable();

        while let Some(change) = iter.next() {
            match change {
                Change::Equal { value } => {
                    self.push(value);
                }

                Change::Delete { value } => {
                    // insert after delete means replace
                    if let Some(Change::Insert { value: next_value }) = iter.peek() {
                        // replacement value is the same as original value, let's skip it
                        if value.to_lowercase() == next_value.to_lowercase() {
                            self.push(next_value);
                        } else {
                            self.merge_two_values(value, next_value);
                        }

                        iter.next();
                    } else {
                        self.push(value)
                    }
                }

                Change::Insert { value } => {
                    self.push(value);
                }
            }
        }
    }
}

struct TextMerger {
    result: String,
}

impl Merger<'_> for TextMerger {
    fn push(&mut self, value: &str) {
        self.result.push_str(value);
    }

    fn merge_two_values(&mut self, left: &'_ str, right: &'_ str) {
        self.push(left);

        if should_add_whitespace(left, right) {
            self.push(" ");
        }

        self.push(right);
    }
}

fn get_word_diff<'s>(left: &'s str, right: &'s str) -> Vec<Change<'s>> {
    let diff = TextDiff::configure()
        .algorithm(Algorithm::Patience)
        .timeout(DEFAULT_MERGE_TIMEOUT)
        .diff_unicode_words(left, right);

    diff.iter_all_changes().map(From::from).collect()
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

pub fn merge_strings_three_way(base: &str, left: &str, right: &str) -> String {
    let max_len = base.len().max(left.len()).max(right.len());

    let mut merger = TextMerger {
        result: String::with_capacity(max_len),
    };

    let left_diff = get_word_diff(base, left);
    let right_diff = get_word_diff(base, right);

    merger.merge_diffs(&left_diff, &right_diff);

    merger.result
}

struct SliceMerger<'s> {
    result: Vec<&'s str>,
}

impl<'s> Merger<'s> for SliceMerger<'s> {
    fn push(&mut self, value: &'s str) {
        self.result.push(value);
    }

    fn push_all(&mut self, values: &[&'s str]) {
        self.result.extend_from_slice(values);
    }

    fn merge_two_values(&mut self, left: &'s str, right: &'s str) {
        self.push(left);
        self.push(right);
    }
}

fn get_slice_diff<'s>(left: &[&'s str], right: &[&'s str]) -> Vec<Change<'s>> {
    let diff = TextDiff::configure()
        .algorithm(Algorithm::Patience)
        .timeout(DEFAULT_MERGE_TIMEOUT)
        .diff_slices(left, right);

    diff.iter_all_changes().map(From::from).collect()
}

pub fn merge_slices_three_way<'s>(
    base: &[&'s str],
    left: &[&'s str],
    right: &[&'s str],
) -> Vec<&'s str> {
    let max_len = base.len().max(left.len()).max(right.len());

    let mut merger = SliceMerger {
        result: Vec::with_capacity(max_len),
    };

    let left_diff = get_slice_diff(base, left);
    let right_diff = get_slice_diff(base, right);

    merger.merge_diffs(&left_diff, &right_diff);

    merger.result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_merge_two_strings_without_base() {
        assert_eq!(
            merge_strings_three_way("", "a good test", "a bad test"),
            "a good bad test"
        );

        assert_eq!(
            merge_strings_three_way("", "a good test", "a  bad test"),
            "a good  bad test"
        );

        assert_eq!(
            merge_strings_three_way("", "a good test says something", "a bad test"),
            "a good bad test says something"
        );

        // first all left, then all right
        assert_eq!(
            merge_strings_three_way("", "test 123", "ok go other"),
            "test 123 ok go other"
        );

        // merge common prefix
        assert_eq!(
            merge_strings_three_way("", "test 123", "test ok go other"),
            "test 123 ok go other"
        );

        // word boundaries
        assert_eq!(
            merge_strings_three_way(
                "",
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

    #[test]
    fn test_merge_slices_three_way() {
        // no changes
        assert_eq!(
            merge_slices_three_way(
                &["The", "quick", "brown"],
                &["The", "quick", "brown"],
                &["The", "quick", "brown"],
            ),
            &["The", "quick", "brown"]
        );

        // non conflicting insertions
        assert_eq!(
            merge_slices_three_way(
                &["The", "quick", "brown"],
                &["The", "slow", "brown"],
                &["The", "quick", "yellow"],
            ),
            &["The", "slow", "yellow"]
        );

        // conflicting insertions
        assert_eq!(
            merge_slices_three_way(
                &["The", "quick", "brown"],
                &["The", "slow", "brown"],
                &["The", "big", "brown"],
            ),
            &["The", "slow", "big", "brown"]
        );

        // conflicting insertions with the same input
        assert_eq!(
            merge_slices_three_way(
                &["The", "quick", "brown"],
                &["The", "slow", "brown"],
                &["The", "slow", "brown"],
            ),
            &["The", "slow", "brown"]
        );

        // deletion in one branch
        assert_eq!(
            merge_slices_three_way(
                &["The", "quick", "brown"],
                &["The", "quick", "brown"],
                &["The", "brown"],
            ),
            &["The", "brown"]
        );

        // conflicting deletion and insertion
        assert_eq!(
            merge_slices_three_way(
                &["The", "quick", "brown"],
                &["The", "quick"],
                &["The", "quick", "brown", "fox"],
            ),
            &["The", "quick", "fox"]
        );

        // completely divergent
        assert_eq!(
            merge_slices_three_way(
                &["The", "quick"], //
                &["The", "slow"],
                &["brown", "quick"],
            ),
            &["brown", "slow"]
        );

        // empty base
        assert_eq!(
            merge_slices_three_way(
                &[], //
                &["The", "slow"],
                &["brown", "quick"],
            ),
            &["The", "slow", "brown", "quick"]
        );

        // empty base, similar edits
        assert_eq!(
            merge_slices_three_way(
                &[], //
                &["The", "slow"],
                &["The", "slow", "brown", "quick"],
            ),
            &["The", "slow", "brown", "quick"]
        );
    }
}
