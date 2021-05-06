use std::collections::HashSet;

use aho_corasick::AhoCorasick;
use anyhow::*;
use rusqlite::Row;

use rs_utils::fill_vec;

use crate::entities::*;

fn extract_refs(value: String) -> serde_json::Result<HashSet<Id>> {
    serde_json::from_str::<HashSet<Id>>(&value)
}

pub fn serialize_refs(refs: &HashSet<Id>) -> serde_json::Result<String> {
    serde_json::to_string(&refs)
}

pub fn extract_document(row: &Row) -> Result<Document> {
    Ok(Document {
        id: row.get("id")?,
        rev: row.get("rev")?,
        prev_rev: row.get("prev_rev")?,
        snapshot_id: row.get("snapshot_id")?,
        document_type: row.get("type")?,
        archived: row.get("archived")?,
        created_at: row.get("created_at")?,
        updated_at: row.get("updated_at")?,
        refs: extract_refs(row.get("refs")?)?,
        data: row.get("data")?,
    })
}

struct SearchScore {
    values: Vec<usize>,
}

impl SearchScore {
    pub fn new(size: usize) -> Self {
        SearchScore {
            values: fill_vec(size, 0),
        }
    }

    pub fn increment_value_at_pos(&mut self, pos: usize, match_pos: usize, str_len: usize) {
        let current_value = self.values[pos];

        // 100pts for the first match, 1pts for next matches
        self.values[pos] = {
            if current_value == 0 {
                100 + SearchScore::get_bonus_points(match_pos, str_len)
            } else {
                current_value + 1
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

    pub fn is_ready(&self) -> bool {
        // each pattern is matched at least once
        self.values.iter().find(|value| **value == 0).is_none()
    }

    pub fn calculate(self) -> u32 {
        if !self.is_ready() {
            return 0;
        }

        self.values
            .into_iter()
            .fold(0, |acc, value| acc + value as u32)
    }
}

pub fn multi_search(pattern: &str, data: &str) -> u32 {
    // cleanup & prepare pattern
    let patterns: Vec<String> = pattern
        .split(" ")
        .map(|item| item.trim().to_lowercase())
        .filter(|item| item.len() > 1)
        .collect();

    let patterns_count = patterns.len();

    if patterns_count == 0 {
        return 1;
    }

    let ac = AhoCorasick::new(patterns);

    let mut score = SearchScore::new(patterns_count);
    for pattern_match in ac.find_iter(data) {
        score.increment_value_at_pos(pattern_match.pattern(), pattern_match.start(), data.len());

        // search only until patterns matched at least once
        if score.is_ready() {
            break;
        }
    }

    score.calculate()
}
