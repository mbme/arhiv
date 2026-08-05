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
use tokenizer::{tokenize_with_offsets, tokenize_with_positions};

use baza_common::log;

use self::document_scorer::DocumentScorer;

// These are common bm25 parameter values
const B: f64 = 0.75;
const K1: f64 = 1.2;

// Bound fuzzy expansion so broad queries do not dilute ranking or scale with vocabulary size.
const MAX_MATCHED_TERMS_PER_QUERY_TERM: usize = 32;

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

// (interned) field -> token position[]; positions are used for proximity scoring.
type DocumentTermMatches = HashMap<FieldId, Vec<usize>>;

#[derive(Default, Serialize, Deserialize)]
pub struct FTSEngine {
    // cache field names
    fields: Vec<String>,

    // term -> document_id -> field -> token position[]
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

            let field_terms = tokenize_with_positions(value);
            if field_terms.is_empty() {
                continue;
            }

            doc_term_count += field_terms.len();

            for (term, token_position) in field_terms {
                let term_matches = self.terms_index.entry(term).or_default();

                let doc_term_matches = term_matches.entry(document_id.clone()).or_default();

                let field_matches = doc_term_matches.entry(field).or_default();
                field_matches.push(token_position);
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
        if self.doc_term_count.is_empty() {
            self.avg_doc_len = 0.0;
            return;
        }

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

    fn get_fuzzy_terms(&self, query_term: &str) -> Vec<TermCandidate<'_>> {
        let mut terms = self
            .terms_index
            .par_iter()
            .filter_map(|(term, _)| {
                if query_term == term {
                    return Some(TermCandidate::exact(term));
                }

                if term.starts_with(query_term) {
                    return Some(TermCandidate::prefix(
                        term,
                        query_term.len() as f64 / term.len() as f64,
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

                    Some(TermCandidate::fuzzy(term, similarity))
                } else {
                    let distance = damerau_levenshtein(query_term, term);
                    if distance > 2 {
                        return None;
                    }

                    let similarity = 1.0 - (0.4 * distance as f64);

                    Some(TermCandidate::fuzzy(term, similarity))
                }
            })
            .collect::<Vec<_>>();

        terms.par_sort_by(|a, b| {
            f64::total_cmp(&b.score_multiplier, &a.score_multiplier)
                .then_with(|| f64::total_cmp(&self.idf(b.term), &self.idf(a.term)))
                .then_with(|| {
                    a.term
                        .len()
                        .abs_diff(query_term.len())
                        .cmp(&b.term.len().abs_diff(query_term.len()))
                })
                .then_with(|| a.term.cmp(b.term))
        });
        terms.truncate(MAX_MATCHED_TERMS_PER_QUERY_TERM);

        terms
    }

    pub fn search(&self, query: &str) -> Vec<&String> {
        let mut seen_query_terms = HashSet::new();
        let query_terms = tokenize_with_offsets(query)
            .into_iter()
            .filter_map(|(term, _)| seen_query_terms.insert(term.clone()).then_some(term))
            .collect::<Vec<_>>();

        // return all the ids in case query is empty
        if query_terms.is_empty() {
            return self.doc_term_count.keys().collect();
        }

        // pick terms that fuzzy match query terms
        // query term -> (fuzzy term, similarity)[]
        let mut all_query_terms = HashMap::new();
        for (query_position, query_term) in query_terms.iter().enumerate() {
            let fuzzy_terms = self.get_fuzzy_terms(query_term);
            if fuzzy_terms.is_empty() {
                log::debug!("Couldn't find terms for query term '{query_term}'");
                return vec![];
            }

            all_query_terms.insert(query_term, (query_position, fuzzy_terms));
        }

        log::debug!(
            "{} query terms -> {} fuzzy matched terms",
            query_terms.len(),
            all_query_terms
                .values()
                .map(|(_, fuzzy_terms)| fuzzy_terms.len())
                .sum::<usize>()
        );

        let mut scores = all_query_terms
            .into_par_iter()
            .flat_map(|(query_term, (query_position, fuzzy_terms))| {
                fuzzy_terms.into_par_iter().flat_map(move |candidate| {
                    let idf = self.idf(candidate.term);

                    let doc_map = self
                        .terms_index
                        .get(candidate.term)
                        .expect("fuzzy matched term must be indexed");

                    doc_map
                        .par_iter()
                        .map(move |(document_id, document_term_matches)| {
                            (
                                query_term,
                                query_position,
                                candidate.score_multiplier,
                                idf,
                                document_id,
                                document_term_matches,
                            )
                        })
                })
            })
            .map(
                |(
                    query_term,
                    query_position,
                    similarity,
                    idf,
                    document_id,
                    document_term_matches,
                )| {
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
                        query_position,
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
                |mut scores,
                 (
                    query_term,
                    query_position,
                    document_id,
                    doc_bm25_score,
                    document_term_matches,
                )| {
                    let document_scorer: &mut DocumentScorer =
                        scores.entry(document_id).or_default();

                    document_scorer.update_term_score(
                        query_term,
                        query_position,
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

    pub fn get_stats(&self) -> FTSStats<'_> {
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

#[derive(Clone, Copy)]
struct TermCandidate<'term> {
    term: &'term str,
    // Keep exact > prefix > fuzzy so approximate matches cannot outrank equally strong exact matches.
    score_multiplier: f64,
}

impl<'term> TermCandidate<'term> {
    fn exact(term: &'term str) -> Self {
        Self {
            term,
            score_multiplier: 1.0,
        }
    }

    fn prefix(term: &'term str, length_ratio: f64) -> Self {
        Self {
            term,
            score_multiplier: 0.8 * length_ratio,
        }
    }

    fn fuzzy(term: &'term str, similarity: f64) -> Self {
        Self {
            term,
            score_multiplier: 0.6 * similarity,
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

    use super::{FTSEngine, MAX_MATCHED_TERMS_PER_QUERY_TERM};

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
    fn test_remove_last_document_resets_average_doc_length() {
        let mut fts = new_test_fts(&[TestDoc::new(1, "title", "data")]);

        fts.remove_document("1");

        assert_eq!(fts.avg_doc_len, 0.0);
        assert!(fts.search("title").is_empty());
    }

    #[test]
    fn test_reindex_document_replaces_old_terms() {
        let mut fts = FTSEngine::new();

        TestDoc::new(1, "old title", "old data").insert(&mut fts);
        TestDoc::new(1, "new title", "new data").insert(&mut fts);

        assert!(fts.search("old").is_empty());
        assert_eq!(fts.search("new"), vec!["1"]);
    }

    #[test]
    fn test_remove_document_preserves_shared_terms_for_other_documents() {
        let mut fts = new_test_fts(&[
            TestDoc::new(1, "shared", "only first"),
            TestDoc::new(2, "shared", "only second"),
        ]);

        fts.remove_document("1");

        assert_eq!(fts.search("shared"), vec!["2"]);
        assert!(fts.search("first").is_empty());
        assert_eq!(fts.search("second"), vec!["2"]);
    }

    #[test]
    fn test_fuzzy_matching_boundaries() {
        let fts = new_test_fts(&[
            TestDoc::new(1, "alpha", "abcd value"),
            TestDoc::new(2, "beta", "cat"),
        ]);

        assert_eq!(fts.search("abc"), vec!["1"]);
        assert!(fts.search("abd").is_empty());
        assert!(fts.search("balue").is_empty());
        assert!(fts.search("catzzz").is_empty());
    }

    #[test]
    fn test_exact_match_beats_prefix_and_fuzzy_matches() {
        let fts = new_test_fts(&[
            TestDoc::new(1, "alpha", "value"),
            TestDoc::new(2, "alpha", "valuable"),
            TestDoc::new(3, "alpha", "vlaue"),
        ]);

        let results = fts.search("value");

        assert_eq!(results[0], "1");
        assert_eq!(results.len(), 3);
    }

    #[test]
    fn test_prefix_match_beats_fuzzy_match() {
        let fts = new_test_fts(&[
            TestDoc::new(1, "alpha", "value"),
            TestDoc::new(2, "alpha", "vblu"),
        ]);

        assert_eq!(fts.search("valu"), vec!["1", "2"]);
    }

    #[test]
    fn test_fuzzy_terms_are_capped() {
        let mut fts = FTSEngine::new();

        for index in 0..64 {
            TestDoc::new(index, "alpha", &format!("value{index}")).insert(&mut fts);
        }

        let fuzzy_terms = fts.get_fuzzy_terms("value");

        assert_eq!(fuzzy_terms.len(), MAX_MATCHED_TERMS_PER_QUERY_TERM);
        assert_eq!(fuzzy_terms[0].term, "value0");
    }

    #[test]
    fn test_multi_term_search_requires_all_terms() {
        let fts = new_test_fts(&[
            TestDoc::new(1, "alpha beta", "gamma"),
            TestDoc::new(2, "alpha", "delta"),
        ]);

        assert_eq!(fts.search("alpha gamma"), vec!["1"]);
        assert!(fts.search("alpha missing").is_empty());
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
