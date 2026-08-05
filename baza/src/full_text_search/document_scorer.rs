use std::collections::HashMap;

use crate::algorithms::smallest_range_covering_elements_from_k_lists;

use super::{DocumentTermMatches, FieldBoost, FieldId};

#[derive(Default)]
pub(super) struct DocumentScorer<'term, 'doc> {
    // query term -> selected term match
    term_matches: HashMap<&'term str, TermMatch<'doc>>,

    // query term -> score
    term_scores: HashMap<&'term str, f64>,
}

struct TermMatch<'doc> {
    query_position: usize,
    matches: &'doc DocumentTermMatches,
}

impl<'term, 'doc> DocumentScorer<'term, 'doc> {
    pub fn terms_count(&self) -> usize {
        self.term_matches.len()
    }

    /// Update score of term, if it's bigger than current score
    pub fn update_term_score(
        &mut self,
        term: &'term str,
        query_position: usize,
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
        self.term_matches.insert(
            term,
            TermMatch {
                query_position,
                matches,
            },
        );
    }

    fn calculate_fields_bonus(&self, field_boosts: &HashMap<FieldId, FieldBoost>) -> f64 {
        let mut bonus = 1.0;

        for (field, field_boost) in field_boosts {
            let terms_in_field = self
                .term_matches
                .values()
                .filter(|term_match| term_match.matches.get(field).is_some())
                .count();

            let field_bonus = field_boost.calculate(terms_in_field, self.terms_count());

            bonus *= field_bonus;
        }

        bonus
    }

    /// Calculate proximity bonus if all the terms matched the field.
    /// Exact phrases outrank ordered near matches, which outrank unordered near matches.
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
            .matches
            .keys()
            .collect::<Vec<_>>();

        let mut max_proximity_bonus = 1.0;
        for field in fields {
            let mut term_field_matches = self
                .term_matches
                .values()
                .filter_map(|term_match| {
                    term_match
                        .matches
                        .get(field)
                        .map(|positions| (term_match.query_position, positions.as_slice()))
                })
                .collect::<Vec<_>>();

            // this field didn't match all terms
            if term_field_matches.len() < self.terms_count() {
                continue;
            }

            term_field_matches.sort_by_key(|(query_position, _)| *query_position);
            let positions_by_query_term = term_field_matches
                .iter()
                .map(|(_, positions)| *positions)
                .collect::<Vec<_>>();

            if has_exact_phrase(&positions_by_query_term) {
                max_proximity_bonus = f64::max(max_proximity_bonus, 2.0);
                continue;
            }

            if let Some(span) = smallest_ordered_span(&positions_by_query_term) {
                let ordered_bonus = (8.0 / (span as f64 + 2.0)).clamp(1.2, 1.6);
                max_proximity_bonus = f64::max(max_proximity_bonus, ordered_bonus);
                continue;
            }

            let (min, max, _) =
                smallest_range_covering_elements_from_k_lists(positions_by_query_term.as_slice());
            let min_distance = max - min;

            let proximity_bonus = (6.0 / (min_distance as f64 + 3.0)).clamp(1.05, 1.25);

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

fn has_exact_phrase(positions_by_query_term: &[&[usize]]) -> bool {
    let Some(first_term_positions) = positions_by_query_term.first() else {
        return false;
    };

    first_term_positions.iter().any(|first_position| {
        positions_by_query_term
            .iter()
            .enumerate()
            .all(|(query_offset, positions)| {
                positions
                    .binary_search(&(first_position + query_offset))
                    .is_ok()
            })
    })
}

fn smallest_ordered_span(positions_by_query_term: &[&[usize]]) -> Option<usize> {
    let first_term_positions = positions_by_query_term.first()?;
    let mut smallest_span = None;

    for first_position in *first_term_positions {
        let mut previous_position = *first_position;

        for positions in positions_by_query_term.iter().skip(1) {
            let next_index = positions.partition_point(|position| *position <= previous_position);
            let Some(next_position) = positions.get(next_index) else {
                previous_position = usize::MAX;
                break;
            };

            previous_position = *next_position;
        }

        if previous_position == usize::MAX {
            continue;
        }

        let span = previous_position - first_position;
        smallest_span = Some(smallest_span.map_or(span, |current: usize| current.min(span)));
    }

    smallest_span
}
