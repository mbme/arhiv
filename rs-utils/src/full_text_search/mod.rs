mod document_scorer;
mod tokenizer;

use std::{
    cmp::Reverse,
    collections::{HashMap, HashSet},
};

use anyhow::{Result, ensure};
use ordermap::OrderMap;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use strsim::damerau_levenshtein;
use tokenizer::tokenize_with_offsets;

use crate::log;

use self::document_scorer::DocumentScorer;

// These are common bm25 parameter values
const B: f64 = 0.75;
const K1: f64 = 1.2;

#[derive(Debug, Serialize, Deserialize)]
#[serde(transparent)]
pub struct FieldBoost(f64);

impl FieldBoost {
    pub fn new(value: f64) -> Result<Self> {
        ensure!(
            (1.0..=2.0).contains(&value),
            "Field boost must be in range [1, 2], got {value}"
        );

        Ok(FieldBoost(value))
    }

    /// calculate bonus for fields proportionally to number of matched query terms in the field
    pub fn calculate(&self, terms_in_field: usize, total_terms_count: usize) -> f64 {
        1.0 + (self.0 - 1.0) * (terms_in_field as f64 / total_terms_count as f64)
    }
}

type FieldId = usize;

// (interned) field -> offset[]
type DocumentTermMatches = HashMap<FieldId, Vec<usize>>;

#[derive(Default, Serialize, Deserialize)]
pub struct FTSEngine {
    // cache field names
    fields: Vec<String>,

    // term -> document_id -> field -> offset[]
    terms_index: HashMap<String, HashMap<String, DocumentTermMatches>>,

    // document_id -> term count
    doc_term_count: HashMap<String, usize>,

    // average term count per document
    avg_doc_len: f64,

    // Boost scores for some document fields
    // document_id -> field -> score_boost
    doc_field_boost: HashMap<String, HashMap<FieldId, FieldBoost>>,
}

impl FTSEngine {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn index_document(
        &mut self,
        document_id: String,
        document: HashMap<&str, &str>,
        boost_scores: HashMap<&str, FieldBoost>,
    ) {
        self.remove_document(&document_id);

        // update term frequency index
        let mut doc_term_count = 0;
        for (field, value) in document {
            let field = self.get_or_intern_field(field);

            let field_terms = tokenize_with_offsets(value);
            if field_terms.is_empty() {
                continue;
            }

            doc_term_count += field_terms.len();

            for (term, byte_offset) in field_terms {
                let term_matches = self.terms_index.entry(term).or_default();

                let doc_term_matches = term_matches.entry(document_id.clone()).or_default();

                let field_matches = doc_term_matches.entry(field).or_default();
                field_matches.push(byte_offset);
            }
        }

        let document_scores = boost_scores
            .into_iter()
            .map(|(key, value)| (self.get_or_intern_field(key), value))
            .collect();
        self.doc_field_boost
            .insert(document_id.clone(), document_scores);

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
        self.doc_field_boost.remove(document_id);

        self.update_avg_doc_term_count();
    }

    fn get_or_intern_field(&mut self, field: &str) -> FieldId {
        if let Some(position) = self.fields.iter().position(|item| item == field) {
            return position;
        }

        self.fields.push(field.to_string());

        self.fields.len() - 1
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
        self.terms_index
            .par_iter()
            .filter_map(|(term, _)| {
                // complete match
                if query_term == term {
                    return Some((term.as_str(), 1.0));
                }

                // match prefixes
                if term.starts_with(query_term) {
                    return Some((
                        term.as_str(),
                        (query_term.len() as f64 / term.len() as f64).min(0.5),
                    ));
                }

                // we need only complete prefix matches for short queries
                if query_term.len() <= 3 {
                    return None;
                }

                // ensure the first letter is the same
                if term.chars().next() != query_term.chars().next() {
                    return None;
                }

                // ensure query term isn't too long to match this term
                if query_term.len() > term.len() + 1 {
                    return None;
                }

                if query_term.len() < term.len() {
                    let distance = damerau_levenshtein(query_term, &term[0..query_term.len()]);
                    if distance > 1 {
                        return None;
                    }

                    let mut similarity = 1.0 - (0.3 * distance as f64);
                    similarity *= query_term.len() as f64 / term.len() as f64;

                    Some((term, similarity))
                } else {
                    let distance = damerau_levenshtein(query_term, term);
                    if distance > 2 {
                        return None;
                    }

                    let similarity = 1.0 - (0.4 * distance as f64);

                    Some((term, similarity))
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

        log::debug!(
            "{} query terms -> {} fuzzy matched terms",
            query_terms.len(),
            all_query_terms
                .values()
                .map(|fuzzy_terms| fuzzy_terms.len())
                .sum::<usize>()
        );

        let mut scores = all_query_terms
            .into_par_iter()
            .flat_map(|(query_term, fuzzy_terms)| {
                fuzzy_terms
                    .into_par_iter()
                    .flat_map(move |(fuzzy_term, similarity)| {
                        let idf = self.idf(fuzzy_term);

                        let doc_map = self
                            .terms_index
                            .get(fuzzy_term)
                            .expect("fuzzy matched term must be indexed");

                        doc_map
                            .par_iter()
                            .map(move |(document_id, document_term_matches)| {
                                (
                                    query_term,
                                    similarity,
                                    idf,
                                    document_id,
                                    document_term_matches,
                                )
                            })
                    })
            })
            .map(
                |(query_term, similarity, idf, document_id, document_term_matches)| {
                    // Calculate BM25 score

                    let doc_len = *self
                        .doc_term_count
                        .get(document_id)
                        .expect("Document term count couldn't be empty")
                        as f64;

                    let tf: f64 = document_term_matches
                        .values()
                        .map(|positions| positions.len() as f64)
                        .sum();
                    let numerator = tf * (K1 + 1.0);
                    let denominator = tf + K1 * (1.0 - B + B * (doc_len / self.avg_doc_len));

                    let doc_bm25_score = idf * (numerator / denominator);

                    // apply fuzzy term similarity coefficient
                    let doc_bm25_score = doc_bm25_score * similarity;

                    (
                        query_term,
                        document_id,
                        doc_bm25_score,
                        document_term_matches,
                    )
                },
            )
            .collect::<Vec<_>>()
            .into_iter()
            .fold(
                HashMap::new(),
                |mut scores, (query_term, document_id, doc_bm25_score, document_term_matches)| {
                    let document_scorer: &mut DocumentScorer =
                        scores.entry(document_id).or_default();

                    document_scorer.update_term_score(
                        query_term,
                        doc_bm25_score,
                        document_term_matches,
                    );

                    scores
                },
            );

        // keep only documents that match all query terms
        scores.retain(|_, document_scorer| document_scorer.terms_count() == query_terms.len());

        let mut result = scores
            .into_iter()
            .map(|(document_id, matches)| {
                (
                    document_id,
                    matches.score(self.doc_field_boost.get(document_id)),
                )
            })
            .collect::<Vec<_>>();

        // sort by score desc
        result.par_sort_by(|a, b| f64::total_cmp(&b.1, &a.1));

        log::debug!("{} search results", result.len());

        result
            .into_iter()
            .map(|(document_id, _)| document_id)
            .collect()
    }

    pub fn get_stats(&self) -> FTSStats {
        let terms_count = self.terms_index.len();
        let docs_count = self.doc_term_count.len();

        let mut terms_usage = self
            .terms_index
            .iter()
            .map(|(term, document_scores)| {
                let term_count = document_scores
                    .values()
                    .flat_map(|term_matches| term_matches.values().map(|offsets| offsets.len()))
                    .sum::<usize>();

                (term.as_str(), term_count)
            })
            .collect::<Vec<_>>();
        terms_usage.sort_by_key(|(_, term_count)| Reverse(*term_count));
        let top_10_terms = terms_usage.into_iter().take(10).collect();

        let mut doc_len = self.doc_term_count.iter().collect::<Vec<_>>();
        doc_len.sort_by_key(|(_, len)| Reverse(*len));
        let top_10_longest_docs = doc_len
            .into_iter()
            .map(|(document_id, &len)| (document_id.as_str(), len))
            .take(10)
            .collect();

        FTSStats {
            top_10_terms,
            top_10_longest_docs,
            terms_count,
            docs_count,
        }
    }
}

#[derive(Debug)]
pub struct FTSStats<'fts> {
    pub top_10_terms: OrderMap<&'fts str, usize>, // term -> term_count
    pub top_10_longest_docs: OrderMap<&'fts str, usize>, // document_id -> term_count
    pub terms_count: usize,
    pub docs_count: usize,
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::full_text_search::FieldBoost;

    use super::FTSEngine;

    #[derive(Clone)]
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

        pub fn insert(&self, engine: &mut FTSEngine) {
            engine.index_document(self.id.clone(), self.get_fields(), Default::default());
        }

        pub fn get_fields(&self) -> HashMap<&str, &str> {
            let mut fields = HashMap::new();
            fields.insert("title", self.title.as_str());
            fields.insert("data", self.data.as_str());

            fields
        }
    }

    fn new_test_fts(docs: &[TestDoc]) -> FTSEngine {
        let mut engine = FTSEngine::new();

        for doc in docs {
            doc.insert(&mut engine);
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

        assert_eq!(fts.search("vlaue").len(), 3);
        assert_eq!(fts.search("titl daata").len(), 3);
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

    #[test]
    fn test_field_boost() {
        let doc1 = TestDoc::new(1, "test value 1", "data 123");
        let doc2 = TestDoc::new(2, "test value 1", "test data 123");

        {
            let fts = new_test_fts(&[doc1.clone(), doc2.clone()]);

            assert_eq!(fts.search("test value"), vec!["2", "1"]);
        }

        {
            let mut fts = FTSEngine::new();

            let mut field_boost = HashMap::new();
            field_boost.insert("title", FieldBoost::new(2.0).unwrap());
            fts.index_document(doc1.id.clone(), doc1.get_fields(), field_boost);

            doc2.insert(&mut fts);

            assert_eq!(fts.search("test value"), vec!["1", "2"]);
        }
    }
}
