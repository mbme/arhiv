mod tokenizer;

use std::collections::{HashMap, HashSet};

use strsim::jaro_winkler;
use tokenizer::tokenize_with_offsets;

use crate::{algorithms::smallest_range_covering_elements_from_k_lists, log};

// These are common bm25 parameter values
const B: f64 = 0.75;
const K1: f64 = 1.2;

const JARO_WINKLER_MIN_SIMILARITY: f64 = 0.8;

type FieldMatches = HashMap<String, Vec<usize>>;

// FIXME simplify lifetime params
#[derive(Default)]
struct DocumentMatches<'query, 'field> {
    // query term -> field -> offset[]
    matches: HashMap<&'query str, &'field FieldMatches>,

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
        matches: &'matches FieldMatches,
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

    /// Calculate proximity bonus if all the terms matched the field.
    /// Returns use max bonus of all the fields.
    fn calculate_proximity_bonus(&self) -> f64 {
        // apply proximity boost if there was more than 1 query term in the document
        if self.terms_matched() < 2 {
            return 1.0;
        }

        let fields = self
            .matches
            .values()
            .next()
            .expect("Matches can't be empty")
            .keys()
            .collect::<Vec<_>>();

        let mut max_proximity_bonus = 1.0;
        for field in fields {
            let term_field_matches = self
                .matches
                .values()
                .filter_map(|field_matches| field_matches.get(field))
                .map(|positions| positions.as_slice())
                .collect::<Vec<_>>();

            // this field didn't match all terms
            if term_field_matches.len() < self.terms_matched() {
                continue;
            }

            let (min, max, _) =
                smallest_range_covering_elements_from_k_lists(term_field_matches.as_slice());
            let min_distance = max - min;

            // Apply an exponential decay function: boost closer matches more
            // boost approaches 2x for very close matches
            let proximity_bonus = (100.0 / (min_distance as f64 + 10.0)).min(2.0);

            max_proximity_bonus = f64::max(max_proximity_bonus, proximity_bonus);
        }

        max_proximity_bonus
    }

    pub fn score(self) -> f64 {
        let proximity_bonus = self.calculate_proximity_bonus();

        self.scores.values().sum::<f64>() * proximity_bonus
    }
}

#[derive(Default)]
pub struct FTSEngine {
    // term -> document_id -> field -> offset[]
    terms_index: HashMap<String, HashMap<String, FieldMatches>>,

    // document_id -> term count
    doc_term_count: HashMap<String, usize>,

    // average term count per document
    avg_doc_len: f64,
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
            if tokens.is_empty() {
                continue;
            }

            doc_term_count += tokens.len();

            for (term, byte_offset) in tokens {
                let term_matches = self.terms_index.entry(term).or_default();

                let document_matches = term_matches.entry(document_id.clone()).or_default();

                let field_matches = document_matches.entry(field.clone()).or_default();
                field_matches.push(byte_offset);
            }
        }

        // update term count index
        *self.doc_term_count.entry(document_id.clone()).or_default() = doc_term_count;

        self.update_avg_doc_term_count();
    }

    pub fn remove_document(&mut self, document_id: &str) {
        self.terms_index.retain(|_, doc_map| {
            // remove entries where key == document_id
            doc_map.retain(|entry_document_id, _| entry_document_id != document_id);

            // remove entry if doc_map becomes empty
            !doc_map.is_empty()
        });

        self.doc_term_count.remove(document_id);

        self.update_avg_doc_term_count();
    }

    fn update_avg_doc_term_count(&mut self) {
        self.avg_doc_len =
            self.doc_term_count.values().sum::<usize>() as f64 / self.doc_term_count.len() as f64;
    }

    fn idf(&self, term: &str) -> f64 {
        let df = self
            .terms_index
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

        self.terms_index
            .keys()
            .filter_map(|term| {
                let similarity = jaro_winkler(query_term, term);

                if similarity > JARO_WINKLER_MIN_SIMILARITY {
                    Some((term.as_str(), similarity))
                } else {
                    None
                }
            })
            .collect()
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

        for (query_term, fuzzy_terms) in all_query_terms {
            for (fuzzy_term, similarity) in fuzzy_terms {
                let doc_map = self
                    .terms_index
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
                    let denominator = tf + K1 * (1.0 - B + B * (doc_len / self.avg_doc_len));

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
            .map(|(document_id, matches)| (document_id, matches.score()))
            .collect::<Vec<_>>();

        // sort by score desc
        result.sort_by(|a, b| f64::total_cmp(&b.1, &a.1));

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
        {
            let fts = new_test_fts(&[
                TestDoc::new(3, "title 3", "test value c asdfdsafasdf 123 data"),
                TestDoc::new(2, "title 2", "data test ok 123"),
                TestDoc::new(1, "title 1", "data 123 test"),
            ]);

            assert_eq!(fts.search("data 123"), vec!["1", "2", "3"]);
        }

        {
            let fts = new_test_fts(&[
                TestDoc::new(1, "title 123", "data test ok else switch"),
                TestDoc::new(
                    2,
                    "title",
                    "data test ok aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa 123",
                ),
            ]);

            assert_eq!(fts.search("data 123"), vec!["2", "1"]);
        }
    }
}
