use std::collections::HashMap;

use crate::algorithms::smallest_range_covering_elements_from_k_lists;

use super::{DocumentTermMatches, FieldBoost, FieldId};

#[derive(Default)]
pub(super) struct DocumentScorer<'term, 'doc> {
    // query term -> field -> offset[]
    term_matches: HashMap<&'term str, &'doc DocumentTermMatches>,

    // query term -> score
    term_scores: HashMap<&'term str, f64>,
}

impl<'term, 'doc> DocumentScorer<'term, 'doc> {
    pub fn terms_count(&self) -> usize {
        self.term_matches.len()
    }

    /// Update score of term, if it's bigger than current score
    pub fn update_term_score(
        &mut self,
        term: &'term str,
        score: f64,
        matches: &'doc DocumentTermMatches,
    ) {
        if let Some(current_score) = self.term_scores.get(term) {
            // we need max score per query term
            if *current_score >= score {
                return;
            }
        }

        self.term_scores.insert(term, score);
        self.term_matches.insert(term, matches);
    }

    fn calculate_fields_bonus(&self, field_boosts: &HashMap<FieldId, FieldBoost>) -> f64 {
        let mut bonus = 1.0;

        for (field, field_boost) in field_boosts {
            let terms_in_field = self
                .term_matches
                .values()
                .filter(|field_matches| field_matches.get(field).is_some())
                .count();

            let field_bonus = field_boost.calculate(terms_in_field, self.terms_count());

            bonus *= field_bonus;
        }

        bonus
    }

    /// Calculate proximity bonus if all the terms matched the field.
    /// Returns max bonus of all the fields.
    fn calculate_proximity_bonus(&self) -> f64 {
        // apply proximity boost if there was more than 1 query term in the document
        if self.terms_count() < 2 {
            return 1.0;
        }

        // list document fields that match ANY term
        // we can take fields for any term (i.e. the first term)
        let fields = self
            .term_matches
            .values()
            .next()
            .expect("Matches can't be empty")
            .keys()
            .collect::<Vec<_>>();

        let mut max_proximity_bonus = 1.0;
        for field in fields {
            let term_field_matches = self
                .term_matches
                .values()
                .filter_map(|field_matches| field_matches.get(field))
                .map(|positions| positions.as_slice())
                .collect::<Vec<_>>();

            // this field didn't match all terms
            if term_field_matches.len() < self.terms_count() {
                continue;
            }

            let (min, max, _) =
                smallest_range_covering_elements_from_k_lists(term_field_matches.as_slice());
            let min_distance = max - min;

            // Apply an exponential decay function: boost closer matches more
            // boost approaches 2x for very close matches
            // min boost is 1.1 since we always want to boost fields that match all query terms
            let proximity_bonus = (100.0 / (min_distance as f64 + 10.0)).clamp(1.1, 2.0);

            max_proximity_bonus = f64::max(max_proximity_bonus, proximity_bonus);
        }

        max_proximity_bonus
    }

    pub fn score(self, field_boosts: Option<&HashMap<FieldId, FieldBoost>>) -> f64 {
        let proximity_bonus = self.calculate_proximity_bonus();

        let fields_bonus = if let Some(field_boosts) = field_boosts {
            self.calculate_fields_bonus(field_boosts)
        } else {
            1.0
        };

        self.term_scores.values().sum::<f64>() * proximity_bonus * fields_bonus
    }
}
