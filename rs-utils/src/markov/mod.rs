use crate::capitalize;
use rand::prelude::*;
use std::collections::HashMap;
use tokenizer::{is_punctuation, Tokenizer};

mod tokenizer;

struct WordStats {
    start: u32,
    end: u32,
    next: HashMap<String, u32>,
}
impl Default for WordStats {
    fn default() -> Self {
        WordStats {
            start: 0,
            end: 0,
            next: HashMap::new(),
        }
    }
}

type WordDistribution = Vec<(String, u32)>;
#[allow(clippy::ptr_arg)]
fn pick_word(distribution: &WordDistribution) -> String {
    distribution
        .choose_weighted(&mut thread_rng(), |(_, prob)| *prob)
        .expect("weights must be vaild")
        .0
        .clone()
}

pub struct Markov {
    starts: WordDistribution,
    ends: WordDistribution,
    separators: WordDistribution,
    words: HashMap<String, WordDistribution>,
}

impl Markov {
    #[must_use]
    pub fn new(text: &str) -> Self {
        let mut markov = Markov {
            starts: vec![],
            ends: vec![],
            separators: vec![],
            words: HashMap::new(),
        };

        markov.train(text);

        markov
    }

    fn train(&mut self, text: &str) {
        let mut separators: HashMap<String, u32> = HashMap::new();
        let mut words: HashMap<String, WordStats> = HashMap::new();

        let sentences = Tokenizer::tokenize(text).get_sentences();

        // 1. count words & separators
        for sentence in sentences {
            *separators.entry(sentence.separator).or_insert(0) += 1;

            let last_pos = sentence.tokens.len() - 1;
            for (pos, word) in sentence.tokens.iter().enumerate() {
                let word_stats = words
                    .entry({
                        if pos == 0 {
                            word.to_string().to_lowercase()
                        } else {
                            word.to_string()
                        }
                    })
                    .or_default();

                if pos == 0 {
                    word_stats.start += 1;
                }

                if pos == last_pos {
                    word_stats.end += 1;
                }

                if pos < last_pos {
                    let next_word = sentence.tokens.get(pos + 1).unwrap().to_string();
                    *word_stats.next.entry(next_word).or_insert(0) += 1;
                }
            }
        }

        // 2. convert into distributions
        for (separator, usages) in separators {
            self.separators.push((separator, usages));
        }

        for (word, word_stats) in words {
            if word_stats.start > 0 {
                self.starts.push((word.clone(), word_stats.start));
            }

            if word_stats.end > 0 {
                self.ends.push((word.clone(), word_stats.end));
            }

            self.words.insert(
                word,
                word_stats
                    .next
                    .into_iter()
                    .map(|(next_word, counter)| (next_word, counter))
                    .collect(),
            );
        }
    }

    #[must_use]
    pub fn generate_sentence(&self, include_separator: bool) -> (String, u32) {
        let mut sentence: Vec<String> = vec![];

        let initial_word = pick_word(&self.starts);
        sentence.push(capitalize(&initial_word));

        let mut word = initial_word;
        loop {
            let distribution = self.words.get(&word).unwrap();
            if distribution.is_empty() {
                break;
            }

            let new_word = pick_word(distribution);

            word = new_word.clone();

            if is_punctuation(&new_word) {
                sentence.iter_mut().last().unwrap().push_str(&new_word);
            } else {
                sentence.push(new_word);
            }

            if word == pick_word(&self.ends) {
                break;
            }
        }

        let words_count = sentence.len();

        let mut result = sentence.join(" ");

        if include_separator {
            result.push_str(&pick_word(&self.separators));
        }

        (result, words_count as u32)
    }

    #[must_use]
    pub fn generate_sentence_constrained(
        &self,
        min_words: u32,
        max_words: u32,
        include_separator: bool,
    ) -> String {
        let mut attempt = 0;
        let max_attempts = 100;

        loop {
            attempt += 1;
            if attempt > max_attempts {
                panic!("Failed to generate sentence in {} attempts", max_attempts);
            }

            let (sentence, words) = self.generate_sentence(include_separator);
            if words >= min_words && words <= max_words {
                return sentence;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_markov() {
        let markov = Markov::new("To begin with, as you may recall from the closing paragraphs of my previous missive, the death of Elly Vaunt shook us all.");

        let (sentence, words) = markov.generate_sentence(true);

        assert!(words > 0);
        assert!(!sentence.is_empty());
    }
}
