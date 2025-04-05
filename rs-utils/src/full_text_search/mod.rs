mod tokenizer;

use std::{
    cmp::Reverse,
    collections::{HashMap, HashSet},
};

use strsim::jaro_winkler;
use tokenizer::tokenize_with_offsets;

use crate::{
    algorithms::{scale_f64_to_u128, smallest_range_covering_elements_from_k_lists},
    log,
};

// These are common bm25 parameter values
const B: f64 = 0.75;
const K1: f64 = 1.2;

const JARO_WINKLER_MIN_SIMILARITY: f64 = 0.8;

struct TermMatch {
    field: String,
    byte_offset: usize,
}

#[derive(Default)]
struct DocumentMatches<'query, 'matches> {
    // query term -> (field, offset)[]
    matches: HashMap<&'query str, &'matches Vec<TermMatch>>,

    // query term -> score
    scores: HashMap<&'query str, f64>,
}

impl<'query, 'matches> DocumentMatches<'query, 'matches> {
    pub fn terms_matched(&self) -> usize {
        self.matches.len()
    }

    /// Update score of term, if it's bigger than current score
    pub fn update_term_score(
        &mut self,
        term: &'query str,
        score: f64,
        matches: &'matches Vec<TermMatch>,
    ) {
        if let Some(current_score) = self.scores.get(term) {
            // we need max score per query term
            if *current_score >= score {
                return;
            }
        }

        self.scores.insert(term, score);
        self.matches.insert(term, matches);
    }

    fn calculate_proximity_bonus(&self) -> f64 {
        // apply proximity boost if there was more than 1 query term in the document
        if self.terms_matched() < 2 {
            return 1.0;
        }

        let arrays = self
            .matches
            .values()
            .map(|matches| {
                matches
                    .iter()
                    .map(|term_match| term_match.byte_offset)
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();

        let (min, max, _) = smallest_range_covering_elements_from_k_lists(arrays.as_slice());
        let min_distance = max - min;

        // Apply an exponential decay function: boost closer matches more
        // boost approaches 2x for very close matches
        let proximity_bonus = (100.0 / (min_distance as f64 + 10.0)).min(2.0);

        proximity_bonus
    }

    fn score(self) -> f64 {
        let proximity_bonus = self.calculate_proximity_bonus();

        self.scores.values().sum::<f64>() * proximity_bonus
    }
}

#[derive(Default)]
pub struct FTSEngine {
    // term -> document_id -> (field, offset)[]
    term_freq_index: HashMap<String, HashMap<String, Vec<TermMatch>>>,

    // document_id -> term count
    doc_term_count: HashMap<String, usize>,
}

impl FTSEngine {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn upsert_document(&mut self, document_id: String, fields: HashMap<String, &str>) {
        self.remove_document(&document_id);

        // update term frequency index
        let mut doc_term_count = 0;
        for (field, value) in fields {
            let tokens = tokenize_with_offsets(value);
            doc_term_count += tokens.len();

            for (term, byte_offset) in tokens {
                let doc_map = self.term_freq_index.entry(term).or_default();

                let matches = doc_map.entry(document_id.clone()).or_default();

                matches.push(TermMatch {
                    field: field.clone(),
                    byte_offset,
                });
            }
        }

        // update term count index
        *self.doc_term_count.entry(document_id.clone()).or_default() = doc_term_count;
    }

    pub fn remove_document(&mut self, document_id: &str) {
        self.term_freq_index.retain(|_, doc_map| {
            // remove entries where key == document_id
            doc_map.retain(|entry_document_id, _| entry_document_id != document_id);

            // remove entry if doc_map becomes empty
            !doc_map.is_empty()
        });

        self.doc_term_count.remove(document_id);
    }

    fn get_avg_doc_term_count(&self) -> f64 {
        self.doc_term_count.values().sum::<usize>() as f64 / self.doc_term_count.len() as f64
    }

    fn idf(&self, term: &str) -> f64 {
        let df = self
            .term_freq_index
            .get(term)
            .map_or(0, |doc_map| doc_map.len());

        if df == 0 {
            return 0.0; // Avoid taking ln(0)
        }

        let n = self.doc_term_count.len();

        ((n as f64 - df as f64 + 0.5) / (df as f64 + 0.5) + 1.0).ln()
    }

    fn get_fuzzy_terms(&self, query_term: &str) -> Vec<(&str, f64)> {
        // FIXME handle 2 chars with starts_with

        let mut result: Vec<(&str, f64)> = self
            .term_freq_index
            .keys()
            .filter_map(|term| {
                let similarity = jaro_winkler(query_term, term);

                if similarity > JARO_WINKLER_MIN_SIMILARITY {
                    Some((term.as_str(), similarity))
                } else {
                    None
                }
            })
            .collect();

        // FIXME remove?
        // order by similarity desc
        result.sort_by(|a, b| f64::total_cmp(&b.1, &a.1));

        result
    }

    pub fn search(&self, query: &str) -> Vec<&String> {
        let query_terms: HashSet<String> = tokenize_with_offsets(query)
            .into_iter()
            .map(|item| item.0)
            .collect();

        // return all the ids in case query is empty
        if query_terms.is_empty() {
            return self.doc_term_count.keys().collect();
        }

        // pick terms that fuzzy match query terms
        // query term -> (fuzzy term, similarity)[]
        let mut all_query_terms = HashMap::new();
        for query_term in &query_terms {
            let fuzzy_terms = self.get_fuzzy_terms(query_term);
            if fuzzy_terms.is_empty() {
                log::debug!("Couldn't find terms for query term '{query_term}'");
                return vec![];
            }

            all_query_terms.insert(query_term, fuzzy_terms);
        }

        let mut scores: HashMap<&String, DocumentMatches> = HashMap::new();

        let avg_doc_len = self.get_avg_doc_term_count();

        for (query_term, fuzzy_terms) in all_query_terms {
            for (fuzzy_term, similarity) in fuzzy_terms {
                let doc_map = self
                    .term_freq_index
                    .get(fuzzy_term)
                    .expect("fuzzy term must be indexed");

                // calculate bm25 for fuzzy term

                let idf = self.idf(fuzzy_term);

                for (document_id, matches) in doc_map {
                    let doc_len = *self
                        .doc_term_count
                        .get(document_id)
                        .expect("Document term count couldn't be empty")
                        as f64;

                    let tf = matches.len() as f64;
                    let numerator = tf * (K1 + 1.0);
                    let denominator = tf + K1 * (1.0 - B + B * (doc_len / avg_doc_len));

                    let doc_bm25_score = idf * (numerator / denominator);

                    // apply fuzzy term similarity coefficient
                    let doc_bm25_score = doc_bm25_score * similarity;

                    let entry = scores.entry(document_id).or_default();
                    entry.update_term_score(query_term, doc_bm25_score, &matches);
                }
            }
        }

        // keep only documents that match all query terms
        scores.retain(|_, document_matches| document_matches.terms_matched() == query_terms.len());

        let mut result = scores
            .into_iter()
            .map(|(document_id, matches)| {
                (
                    document_id,
                    // FIXME remove this, sort by f64
                    scale_f64_to_u128(matches.score())
                        .expect("Score must be finite & non-negative"),
                )
            })
            .collect::<Vec<_>>();
        result.sort_by_cached_key(|(_, score)| Reverse(*score));

        result
            .into_iter()
            .map(|(document_id, _)| document_id)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::FTSEngine;

    struct TestDoc {
        id: String,
        title: String,
        data: String,
    }

    impl TestDoc {
        pub fn new(id: usize, title: &str, data: &str) -> Self {
            TestDoc {
                id: id.to_string(),
                title: title.into(),
                data: data.into(),
            }
        }
    }

    fn new_test_fts(docs: &[TestDoc]) -> FTSEngine {
        let mut engine = FTSEngine::new();

        for doc in docs {
            let mut fields = HashMap::new();
            fields.insert("title".to_string(), doc.title.as_str());
            fields.insert("data".to_string(), doc.data.as_str());

            engine.upsert_document(doc.id.clone(), fields);
        }

        engine
    }

    #[test]
    fn test_search() {
        let fts = new_test_fts(&[
            TestDoc::new(1, "title 1", "data value a"),
            TestDoc::new(2, "title 2", "data value b"),
            TestDoc::new(3, "title 3", "data value cde"),
        ]);

        assert_eq!(fts.search("title").len(), 3);
        assert_eq!(fts.search("title cd").len(), 1);
        assert_eq!(fts.search(" ").len(), 3);

        assert_eq!(fts.search("vlue").len(), 3);
        assert_eq!(fts.search("tetl daaata").len(), 3);
        assert_eq!(fts.search("tit").len(), 3);
    }

    #[test]
    fn test_proximity_boost() {
        let fts = new_test_fts(&[
            TestDoc::new(3, "title 3", "test value c asdfdsafasdf 123 data"),
            TestDoc::new(2, "title 2", "data test ok 123"),
            TestDoc::new(1, "title 1", "data 123 test"),
        ]);

        assert_eq!(fts.search("data 123"), vec!["1", "2", "3"]);
    }
}
