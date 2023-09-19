use aho_corasick::AhoCorasick;

struct SearchScore {
    values: Vec<usize>,
}

impl SearchScore {
    pub fn new(size: usize) -> Self {
        SearchScore {
            values: vec![0; size],
        }
    }

    pub fn increment_value_at_pos(&mut self, pos: usize, match_pos: usize, str_len: usize) {
        let current_value = self.values[pos];

        // 100pts for the first match, 10pts for the following matches
        self.values[pos] = {
            if current_value == 0 {
                100 + SearchScore::get_bonus_points(match_pos, str_len)
            } else {
                current_value + 10
            }
        };
    }

    // calculate bonus points for position, 0 to 20, log scale
    // https://www.wolframalpha.com/input/?i=20+*+%281+-+1%2F2log10%28x%29%29+from+1+to+100
    fn get_bonus_points(match_pos: usize, str_len: usize) -> usize {
        let match_pos_percent: f64 = (match_pos as f64 / str_len as f64) * 100.0;
        let match_pos_percent = (match_pos_percent + 1.0).min(100.0); // renormalize from 1.0 to 100.0

        let bonus_points = 20.0 * (1.0 - 0.5 * match_pos_percent.log10());

        bonus_points.round() as usize
    }

    /// if each pattern is matched at least once
    #[inline]
    fn is_ready(&self) -> bool {
        !self.values.iter().any(|value| *value == 0)
    }

    pub fn calculate(self) -> usize {
        if !self.is_ready() {
            return 0;
        }

        self.values.into_iter().sum()
    }
}

pub struct MultiSearch {
    patterns_count: usize,
    ac: AhoCorasick,
}

impl MultiSearch {
    pub fn new(pattern: impl AsRef<str>) -> Self {
        let patterns: Vec<_> = pattern
            .as_ref()
            .split(' ')
            .map(|item| item.trim().to_lowercase())
            .filter(|item| item.len() > 1)
            .collect();

        MultiSearch {
            patterns_count: patterns.len(),
            ac: AhoCorasick::new(patterns).expect("pattern must be valid"),
        }
    }

    pub fn search(&self, data: &str) -> usize {
        if self.patterns_count == 0 {
            return 1;
        }

        let mut score = SearchScore::new(self.patterns_count);
        for pattern_match in self.ac.find_iter(data) {
            score.increment_value_at_pos(
                pattern_match.pattern().as_usize(),
                pattern_match.start(),
                data.len(),
            );
        }

        score.calculate()
    }
}
