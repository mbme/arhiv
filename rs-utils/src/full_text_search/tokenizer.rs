use charabia::Tokenize;
use unicode_normalization::UnicodeNormalization;

pub fn tokenize_with_offsets(input: &str) -> Vec<(String, usize)> {
    // TODO remove stop words
    input
        .tokenize()
        .filter_map(|token| {
            token
                .is_word()
                .then(|| (token.lemma().nfc().collect(), token.byte_start))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::tokenize_with_offsets;

    #[test]
    fn test_tokenize_with_offsets() {
        assert_eq!(
            tokenize_with_offsets("Hello, 世界! Rust."),
            vec![
                ("hello".to_string(), 0),
                ("世界".to_string(), 7),
                ("rust".to_string(), 15)
            ]
        );

        assert_eq!(
            tokenize_with_offsets("Café naïve façade résumé"),
            vec![
                ("cafe".to_string(), 0),
                ("naive".to_string(), 6),
                ("facade".to_string(), 13),
                ("resume".to_string(), 21)
            ]
        );

        assert_eq!(
            tokenize_with_offsets("ТеСт ЇЖак"),
            vec![
                ("тест".to_string(), 0), //
                ("їжак".to_string(), 9),
            ]
        );
    }
}
