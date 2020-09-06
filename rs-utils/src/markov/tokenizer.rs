use std::fmt;

static ABBREVIATIONS: [&str; 4] = ["dr.", "mr.", "mrs.", "phd."];
static SEPARATORS: [char; 3] = ['.', '?', '!'];
static PUNCTUATION: [char; 3] = [',', ';', ':'];

pub fn is_punctuation(s: &str) -> bool {
    if s.len() != 1 {
        return false;
    }

    let first_char = s.chars().next().unwrap();

    return PUNCTUATION.contains(&first_char);
}

#[derive(Debug)]
pub enum TextToken<'a> {
    Word(&'a str),
    Abbr(String),
    Newline,
    Whitespace,
    Punctuation(char),
    Separator(String),
}

impl<'a> fmt::Display for TextToken<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TextToken::Word(s) => write!(f, "{}", s),
            TextToken::Abbr(a) => write!(f, "{}", a),
            TextToken::Newline => write!(f, "\n"),
            TextToken::Whitespace => write!(f, " "),
            TextToken::Punctuation(c) => write!(f, "{}", c),
            TextToken::Separator(s) => write!(f, "{}", s),
        }
    }
}

pub struct Sentence<'a> {
    pub tokens: Vec<TextToken<'a>>,
    pub separator: String,
}

#[cfg(test)]
impl<'a> Sentence<'a> {
    fn into_vec(self) -> Vec<String> {
        let mut result: Vec<String> = self
            .tokens
            .into_iter()
            .map(|token| token.to_string())
            .collect();

        result.push(self.separator);

        result
    }
}

pub struct Tokenizer<'a> {
    text: &'a str,
    tokens: Vec<TextToken<'a>>,
    word_start: usize,
    pos: usize,
}

impl<'a> Tokenizer<'a> {
    fn new(text: &'a str) -> Self {
        Tokenizer {
            text,
            tokens: vec![],
            word_start: 0,
            pos: 0,
        }
    }

    fn add_token(&mut self, token: TextToken<'a>) {
        self.tokens.push(token);
        self.word_start = self.pos + 1;
    }

    fn is_word_pending(&self) -> bool {
        self.word_start < self.pos
    }

    fn get_pending_word(&self) -> &'a str {
        &self.text[self.word_start..self.pos]
    }

    fn save_pending_word(&mut self) {
        if self.is_word_pending() {
            self.add_token(TextToken::Word(self.get_pending_word()));
        }
    }

    fn skip_char(&mut self) {
        self.word_start = self.pos + 1;
    }

    fn last_token(&mut self) -> Option<&mut TextToken<'a>> {
        self.tokens.iter_mut().last()
    }

    fn normalize(&mut self) {
        let tokens = std::mem::replace(&mut self.tokens, vec![]);

        for (pos, token) in tokens.into_iter().enumerate() {
            if pos == 0 {
                self.tokens.push(token);
                continue;
            }

            let last_token = self.last_token().unwrap();

            match (&token, last_token) {
                // skip duplicate punctuation
                (TextToken::Punctuation(_), TextToken::Punctuation(_)) => {
                    continue;
                }
                // skip duplicate whitespace
                (TextToken::Whitespace, TextToken::Whitespace) => {
                    continue;
                }
                // skip duplicate newline
                (TextToken::Newline, TextToken::Newline) => {
                    continue;
                }
                // concat sequential separators
                (
                    TextToken::Separator(current_separator),
                    TextToken::Separator(previous_separator),
                ) => {
                    previous_separator.push_str(current_separator);
                    continue;
                }
                _ => {}
            }

            self.tokens.push(token);
        }

        // make sure text ends with separator
        let last_token = self.tokens.iter().last().unwrap();
        match last_token {
            TextToken::Separator(_) => {}
            _ => {
                self.tokens.push(TextToken::Separator(".".to_string()));
            }
        }
    }

    pub fn get_sentences(self) -> Vec<Sentence<'a>> {
        let mut sentences = vec![];

        let mut sentence = vec![];
        for token in self.tokens {
            match token {
                TextToken::Newline | TextToken::Whitespace => {}
                TextToken::Separator(_) => {
                    sentence.push(token);
                    sentences.push(sentence);
                    sentence = vec![];
                }
                _ => {
                    sentence.push(token);
                }
            }
        }

        if !sentence.is_empty() {
            sentences.push(sentence);
        }

        sentences
            .into_iter()
            .map(|tokens| {
                let separator = {
                    let token = tokens.iter().last().unwrap();

                    if let TextToken::Separator(separator) = token {
                        separator
                    } else {
                        panic!("last token of the sentence isn't separator")
                    }
                };

                Sentence {
                    separator: separator.to_string(),
                    tokens: tokens
                        .into_iter()
                        .filter(|token| match token {
                            TextToken::Newline
                            | TextToken::Whitespace
                            | TextToken::Separator(_) => false,
                            _ => true,
                        })
                        .collect(),
                }
            })
            .collect()
    }

    pub fn tokenize(text: &'a str) -> Tokenizer<'a> {
        let mut tokenizer = Tokenizer::new(text);

        for (pos, char) in text.trim().char_indices() {
            tokenizer.pos = pos;

            // check if abbr
            if char == '.' && tokenizer.is_word_pending() {
                let mut abbr = tokenizer.get_pending_word().to_lowercase();
                abbr.push(char);

                if ABBREVIATIONS.contains(&abbr.as_str()) {
                    tokenizer.add_token(TextToken::Abbr(abbr));
                    continue;
                }
            }

            if SEPARATORS.contains(&char) {
                tokenizer.save_pending_word();
                tokenizer.add_token(TextToken::Separator(String::from(char)));
                continue;
            }

            if PUNCTUATION.contains(&char) {
                tokenizer.save_pending_word();
                tokenizer.add_token(TextToken::Punctuation(char));
                continue;
            }

            if char == '\r' {
                // ignore it, report only \n
                tokenizer.save_pending_word();
                tokenizer.skip_char();
                continue;
            }

            if char == '\n' {
                tokenizer.save_pending_word();
                tokenizer.add_token(TextToken::Newline);
                continue;
            }

            if char.is_whitespace() {
                tokenizer.save_pending_word();
                tokenizer.add_token(TextToken::Whitespace);
                continue;
            }
        }

        // save last word if any
        tokenizer.pos += 1;
        tokenizer.save_pending_word();

        tokenizer.normalize();

        tokenizer
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenizer() {
        assert_eq!(
            Tokenizer::tokenize("Split   it,or not; dr. go!! test")
                .tokens
                .iter()
                .filter(|item| {
                    if let TextToken::Whitespace = item {
                        false
                    } else {
                        true
                    }
                })
                .map(|item| item.to_string())
                .collect::<Vec<String>>(),
            vec!["Split", "it", ",", "or", "not", ";", "dr.", "go", "!!", "test", "."]
        );
    }

    #[test]
    fn test_sentences() {
        let sentences = Tokenizer::tokenize("Test sentence kedr...Is it? oh").get_sentences();

        assert_eq!(
            sentences
                .into_iter()
                .map(|sentence| sentence.into_vec())
                .collect::<Vec<_>>(),
            vec![
                vec!["Test", "sentence", "kedr", "..."],
                vec!["Is", "it", "?"],
                vec!["oh", "."],
            ]
        );
    }
}
