# Full-Text Search Specification

Status: intended durable search model

## Purpose and scope

Full-text search helps the Arhiv owner find records by indexed text fields. It is a local, deterministic ranking feature over one Arhiv's current search index.

This document governs query normalization, result eligibility, candidate matching, ranking, and search-index compatibility. It does not govern UI presentation, storage encryption, document schema validation, merge behavior, or cross-device synchronization.

## Usage constraints

Search is optimized for navigational record lookup and record-picking flows. The primary expected interaction is a short query that quickly narrows records so the owner can open, reference, or select one of the top results.

Ranking should prioritize first-page precision over broad recall. Exact and prefix matches on identifying fields, especially title and id, should rank ahead of weaker fuzzy or body-field matches.

Search is also used by catalog views, but catalog browsing uses the same deterministic strict-AND semantics. The engine must not silently switch to relaxed, exploratory, OR, semantic, or recommendation-style behavior for catalog usage.

## Indexed content

The search index includes:

- record title text;
- record id text; and
- schema fields whose field type exposes searchable string data.

The search index does not expand references into referenced record titles. Reference-title indexing is out of scope unless a future spec update explicitly adds it.

## Query normalization

Search uses the same token normalization model for indexed content and query text:

1. tokenize text into word tokens;
2. use token lemmas provided by the tokenizer;
3. transliterate tokens to ASCII-compatible text;
4. lowercase tokens; and
5. deduplicate repeated query terms while preserving first occurrence order.

Stop-word removal is out of scope. Query terms that normalize to common words remain ordinary required terms.

An empty normalized query matches every indexed record.

## Result eligibility

Search uses strict AND semantics.

A non-empty query result is eligible only when the record matches every normalized query term. A record may satisfy a query term through an exact, prefix, or fuzzy candidate term match.

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

Eligible records are ranked by a lexical score derived from BM25.

For each query term, the engine scores candidate term matches against each record and keeps the best-scoring candidate for that query term in that record. Candidate scoring is field-aware: the best matching indexed field for that candidate contributes the candidate's per-query-term score. The record's lexical score is the sum of these best per-query-term scores.

Candidate match quality is part of the lexical score. Exact matches receive the strongest multiplier, prefix matches receive a weaker multiplier, and fuzzy matches receive the weakest multiplier.

## Field boosts

Field boosts are bounded ranking multipliers applied during field-aware per-term scoring.

Title and id fields receive explicit boosts because they identify a record more directly than ordinary body fields. Field boosts must not make weak lexical matches dominate clearly better exact matches in ordinary fields.

Additional schema/type-specific boosts require an explicit spec update. Field boost rules should remain centralized instead of spreading product ranking rules through callers.

## Proximity and phrase boosts

The search index stores token positions for each term occurrence. Proximity ranking is based on token positions, not byte offsets.

When all query terms match the same indexed field, the engine may apply one proximity boost for that field. The highest field-level proximity boost is used for the record.

Proximity quality is ordered as:

```text
exact ordered phrase > ordered near match > unordered near match > no proximity boost
```

An exact ordered phrase means the matched tokens appear contiguously in query-term order. An ordered near match means the matched tokens appear in query-term order but are not contiguous. An unordered near match means all query terms appear in a compact token span without preserving query order.

Proximity boosts must remain bounded so they improve ordering among eligible records without overriding strict eligibility or overwhelming lexical relevance.

## Search-index compatibility

Search-index serialization is an implementation detail, but persisted indexes must be invalidated when the indexed data model or ranking-critical stored data changes.

Changing stored term positions, token normalization, candidate classes, or ranking semantics requires bumping the search algorithm version so stale indexes are rebuilt instead of reused silently.

## Non-goals

The following are intentionally out of scope for this specification:

- OR search or relaxed partial-term fallback;
- stop-word removal;
- referenced-record title expansion;
- synonym expansion;
- semantic/vector search;
- exploratory web-search-style ranking or recommendation behavior;
- remote search services; and
- UI-specific result grouping or highlighting.
