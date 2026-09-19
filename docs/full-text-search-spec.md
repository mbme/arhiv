# Full-Text Search Specification

## Purpose and scope

Full-text search helps the Arhiv owner find documents by indexed text fields. It
is a local, deterministic ranking feature over one Arhiv's current search index.

This document explains the current query normalization, result eligibility,
candidate matching, ranking, and search-index compatibility behavior. UI
presentation, storage encryption, document schema validation, merge behavior,
and cross-device synchronization are outside its scope.

## Usage constraints

Search is optimized for navigational document lookup and document-picking
flows. The primary expected interaction is a short query that quickly narrows
documents so the owner can open, reference, or select one of the top results.

Ranking should prioritize first-page precision over broad recall. Exact and prefix matches on identifying fields, especially title and id, should rank ahead of weaker fuzzy or body-field matches.

Search is also used by catalog views, but catalog browsing uses the same deterministic strict-AND semantics. The engine must not silently switch to relaxed, exploratory, OR, semantic, or recommendation-style behavior for catalog usage.

## Indexed content

The search index includes:

- document title text;
- document id text; and
- schema fields whose field type exposes searchable string data.

The search index does not expand references into referenced document titles.

## Query normalization

Search uses the same token normalization model for indexed content and query text:

1. tokenize text into word tokens;
2. use token lemmas provided by the tokenizer;
3. transliterate tokens to ASCII-compatible text;
4. lowercase tokens; and
5. deduplicate repeated query terms while preserving first occurrence order.

Stop-word removal is out of scope. Query terms that normalize to common words remain ordinary required terms.

An empty normalized query matches every indexed document.

## Result eligibility

Search uses strict AND semantics.

A non-empty query result is eligible only when the document matches every
normalized query term. A document may satisfy a query term through an exact,
prefix, or fuzzy candidate term match.

If any query term has no candidate indexed terms, search returns no results.

Search must not fall back to OR, partial coverage, or relaxed matching when strict AND returns no results.

## Candidate term matching

For each normalized query term, the engine may consider these candidate indexed term classes:

1. exact candidate: the indexed term equals the query term;
2. prefix candidate: the indexed term starts with the query term; and
3. fuzzy candidate: the indexed term is close enough to the query term under the configured edit-distance rules.

Candidate quality must be ordered as:

```text
exact > prefix > fuzzy
```

Fuzzy matching is intentionally conservative for short query terms. Short query terms should prefer exact or prefix matching to avoid noisy results.

Fuzzy prefix matching may account for one omitted or extra character across the query/indexed-term prefix boundary so typo recovery can match a short misspelled query against a longer indexed term.

Candidate expansion must be bounded per query term. When there are more candidate indexed terms than the configured cap, the engine keeps the best candidates by match quality, inverse document frequency, and term-length closeness.

When exact or prefix candidates exist for a query term, the engine should prefer those navigational candidates and may discard fuzzy candidates for that term. Fuzzy matching is a typo-recovery mechanism, not a broad recall mechanism.

## Base ranking

Eligible documents are ranked by a lexical score derived from BM25.

For each query term, the engine scores candidate term matches against each
document and keeps the best-scoring candidate for that query term in that
document. Candidate scoring is field-aware: the best matching indexed field for
that candidate contributes the candidate's per-query-term score. The document's
lexical score is the sum of these best per-query-term scores.

BM25 length normalization uses the matched field's token count and that field's
average token count across indexed documents. A long ordinary field must not
reduce the score of a concise title or id match in the same document.

Candidate match quality is part of the lexical score. Exact matches receive the strongest multiplier, prefix matches receive a weaker multiplier, and fuzzy matches receive the weakest multiplier.

## Field boosts

Field boosts are bounded ranking multipliers applied during field-aware per-term scoring.

Title and id fields receive explicit boosts because they identify a document
more directly than ordinary body fields. Field boosts must not make weak lexical
matches dominate clearly better exact matches in ordinary fields.

Title and id are the only specially boosted fields. Their boost values are
assigned centrally while documents are indexed.

## Proximity and phrase boosts

The search index stores token positions for each term occurrence. Proximity ranking is based on token positions, not byte offsets.

When all query terms match the same indexed field, the engine may apply one
proximity boost for that field. The highest field-level proximity boost is used
for the document.

Proximity quality is ordered as:

```text
exact ordered phrase > ordered near match > unordered near match > no proximity boost
```

An exact ordered phrase means the matched tokens appear contiguously in query-term order. An ordered near match means the matched tokens appear in query-term order but are not contiguous. An unordered near match means all query terms appear in a compact token span without preserving query order.

Proximity boosts must remain bounded so they improve ordering among eligible
documents without overriding strict eligibility or overwhelming lexical
relevance.

## Search-index compatibility

The persisted search index records a format version, search algorithm version,
schema data version, and schema fingerprint. The current search algorithm
version is `5`.

Index loading rejects a mismatch in any of these values. Arhiv then rebuilds
the index from current document state instead of silently reusing incompatible
ranking data.

## Non-goals

The following are intentionally out of scope for this specification:

- OR search or relaxed partial-term fallback;
- stop-word removal;
- referenced-document title expansion;
- synonym expansion;
- semantic/vector search;
- exploratory web-search-style ranking or recommendation behavior;
- remote search services; and
- UI-specific result grouping or highlighting.