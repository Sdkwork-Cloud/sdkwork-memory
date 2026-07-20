# Memory Commercial Retrieval Hardening

Status: active
Owner: SDKWork maintainers
Updated: 2026-07-20
Specs: `CODE_STYLE_SPEC.md`, `RUST_CODE_SPEC.md`, `PRIVACY_SPEC.md`, `TEST_SPEC.md`

## 1. Outcome

SDKWork Memory has a credible commercial control plane and a production-capable
embedding-optional data path. Before this hardening, however, the final ranking
stage was still a Phase 1 heuristic: provider scores were added even though
their scales were not comparable, duplicate candidates were not fused, and the
context pack could include a fragment larger than its declared token budget.

This change upgrades the provider-independent path without changing public API,
canonical storage, or generated SDK contracts:

- independently ranked retriever outputs are combined with weighted reciprocal
  rank fusion (RRF), using rank constant `60`;
- duplicate hits from the same retriever cannot amplify a memory, while agreement
  across different retrievers increases its fused score;
- the service retrieves a bounded `4 * topK` candidate pool, capped by the SPI
  limit of `200`, before final fusion;
- recency is a tie-breaking signal with a seven-day half-life instead of a
  one-day inverse decay that suppressed durable semantic memory;
- every returned hit records the dominant and contributing retrievers;
- retrieval profiles execute a validated `weighted_rrf` fusion policy with a
  configurable rank constant from `1` through `1000`, instead of storing an
  ignored policy document;
- context assembly enforces its token budget, safely truncates an oversized
  first fragment, suppresses near duplicates, and estimates CJK text by
  characters rather than UTF-8 bytes.
- runtime selection is typed and fail-closed: storage selects `native_sql` or
  `local_embedded`, while retrieval selects `balanced`, `search_first`, or
  `event_aware`; unknown or storage-incompatible selections stop startup;
- evaluation-only implementation families cannot become `primary`, cannot be
  promoted by migration, and must be stored as disabled shadow metadata with
  `productionQualified=false`.
- the reference plugin advertises only its executable keyword reference path;
  placeholder graph, provider, index, and evaluation ports are no longer
  bindable, and direct legacy calls fail closed;
- `retrieval_quality` runs execute the same retrieval path as production over
  bounded inline golden cases and report macro Recall@K, Precision@K, Hit
  Rate@K, MRR, mean binary nDCG@K, degraded rate, nearest-rank p95 latency,
  per-case query hashes, and optional operator-defined gates;
- consolidation uses transactional, identity-bounded supersession instead of
  labeling soft deletion as a merge; records from different users, scopes,
  memory types, or sensitivity classes cannot be consolidated together;
- exact duplicate consolidation transfers non-conflicting source evidence,
  removes duplicate source links, recomputes the winner evidence count from
  persisted rows, removes stale SQLite FTS projections, and writes one
  `memory.record.superseded` outbox event plus audit record in the same
  transaction; stable operation ids make committed results retry-recoverable;
- retrieval trace query hashes use normalized SHA-256 rather than Rust's
  non-cryptographic, implementation-defined default hasher.

The implementation remains deterministic and embedding optional. Vector,
graph, and model rerank providers can be added behind existing boundaries after
their production qualification and evaluation gates are complete.

## 2. Current Implementation Evidence

| Capability | Status | Evidence |
| --- | --- | --- |
| Canonical tenant and space scoped records | production | `ai_record`, scoped SPI queries, canonical rehydration in `OpenMemoryService` |
| PostgreSQL and SQLite lexical retrieval | production | PostgreSQL `tsvector`, SQLite FTS5, bounded fallback in `sdkwork-memory-plugin-native-sql` |
| Keyword, dictionary, structured SQL, time, and linked-event signals | production | `sdkwork-memory-retrieval` orchestration and native SQL candidate search |
| Cross-retriever score fusion | hardened | weighted RRF, per-retriever deduplication, deterministic tie breaking |
| Context pack budget and redundancy control | hardened | ranked budget selection, safe truncation, Jaccard near-duplicate suppression |
| Tenant authorization and sensitivity filtering | production | authorization snapshot, store-level sensitivity predicate, canonical recheck |
| Retrieval trace and explanation | production | `ai_retrieval_trace`, `ai_retrieval_hit`, fusion explanation JSON |
| Candidate learning, habits, feedback, audit, outbox, quota | production baseline | service and native SQL plugin ports with contract tests |
| Exact canonical duplicate consolidation | hardened | identity-bounded supersession, source transfer/deduplication, persisted evidence recount, FTS cleanup, atomic outbox/audit, operation retry recovery |
| Vector retrieval | contract only | SPI types and implementation profiles exist; no qualified provider is active |
| Model reranking | contract only | `RerankModelPort` exists; production request/score contract is not sufficient yet |
| Graph-temporal retrieval | data/control-plane partial | entity and edge management exists; no production graph retriever is active |
| External memory bridge | evaluation only | reference profile is explicitly evaluation-only |
| Offline retrieval quality evaluation | production baseline | bounded inline golden cases execute production retrieval and calculate Recall@K, macro Precision@K, Hit Rate@K, MRR, mean binary nDCG@K, degradation, real monotonic-clock p95 latency, and optional gates |
| Versioned dataset registry and release promotion gate | not implemented | `datasetRef` is persisted as identity metadata; external dataset resolution and automated release promotion remain fail-closed |

## 2.1 Commercial Scheme Catalog

Commercial schemes are compositions of a qualified canonical store and a
materialized retrieval strategy. A strategy name is never treated as proof of a
separate store or provider implementation.

| Scheme | Canonical store | Retrieval behavior | Qualification |
| --- | --- | --- | --- |
| `native-balanced-v1` | PostgreSQL native SQL | keyword + dictionary + structured SQL + time + linked event | server/container production |
| `native-search-first-v1` | PostgreSQL native SQL | PostgreSQL FTS candidate recall + keyword/dictionary/structured fusion | server/container production |
| `native-event-aware-v1` | PostgreSQL native SQL | linked event evidence prioritized with lexical and recency support | server/container production |
| `local-balanced-v1` | SQLite local embedded | SQLite FTS5 plus all native signals | local/private/test |
| `local-search-first-v1` | SQLite local embedded | SQLite FTS5-first lexical retrieval | local/private/test |
| `local-event-aware-v1` | SQLite local embedded | linked local event evidence plus lexical and recency support | local/private/test |

Startup selection uses safe private process config:

```text
SDKWORK_MEMORY_IMPLEMENTATION_PROFILE=auto|native_sql|local_embedded
SDKWORK_MEMORY_RETRIEVAL_STRATEGY=balanced|search_first|event_aware
```

`auto` maps PostgreSQL to `native_sql` and SQLite to `local_embedded`.
`native_sql` with SQLite and `local_embedded` with PostgreSQL are rejected.
Changing startup selection requires restart; a request may still select an
approved tenant retrieval profile through `retrievalProfileId`.

The following implementation kinds remain evaluation-only until their required
ports and conformance evidence exist: `event_sourced`, `search_first` as a
separate store implementation, `graph_temporal`, `external_provider_bridge`,
and `hybrid_platform`. The production `*-search-first-v1` schemes above are
explicitly native SQL plus an FTS-first retrieval strategy, not those unbuilt
implementation families.

This table is intentionally conservative. A table, DTO, port, or profile name is
not evidence that its algorithm is production qualified.

## 3. Industry Alignment

Professional memory systems converge on a layered design rather than one
universal store:

1. A canonical, auditable record layer preserves provenance, ownership,
   corrections, deletion, and replay.
2. A write pipeline extracts facts and episodes, normalizes entities, detects
   duplicates and contradictions, and promotes durable memories under policy.
3. Multiple derived indexes provide sparse lexical, dense vector, graph,
   temporal, and event recall without becoming canonical truth.
4. Rank fusion combines heterogeneous recall lists; a query-aware reranker can
   refine the bounded head of the list.
5. Context assembly optimizes relevance, diversity, citation quality, privacy,
   and token cost.
6. Offline golden datasets and online feedback measure Recall@K, nDCG@K, MRR,
   context precision, answer support, latency, and cost before rollout.

SDKWork Memory now has strong coverage of layers 1, 3 (sparse), 4 (deterministic
fusion), and 5, plus a real bounded offline retrieval evaluation baseline. The
remaining quality gap is a versioned dataset registry and automated release
promotion workflow that proves each profile is better for a declared workload
and code revision.

Weighted RRF was selected because lexical, event, graph, vector, and reranker
scores are not calibrated to the same scale. Rank-based fusion is robust to
that mismatch and is widely used in hybrid search. For a memory `d`:

```text
rrf(d) = sum(weight(retriever) * relevance(d) / (60 + rank(retriever, d)))
```

The implementation maps the positive aggregate monotonically into `[0, 1)` for
the public fused score. Ordering uses the unrounded aggregate, then raw score,
then memory id for deterministic replay.

## 4. Commercial Gaps And Required Next Gates

### P0: Versioned Evaluation Before Broader Algorithm Claims

- Create versioned, tenant-safe golden datasets for preference, episodic,
  procedural, relationship, contradiction, multilingual, and long-horizon
  memory queries.
- Resolve `datasetRef` through a reviewed dataset registry. Until that resolver
  exists, `retrieval_quality` requires bounded inline `config.cases` and fails
  rather than pretending that the reference was evaluated.
- Extend binary relevance cases to reviewed graded relevance judgments before
  claiming graded nDCG; the current implementation reports honest binary
  nDCG@K, macro returned-hit Precision@K, and nearest-rank p95 latency measured
  around the production retrieval call.
- Add provider-attributed error rate and cost per retrieval only after real
  provider telemetry and billing sources exist; do not synthesize either value.
- Add shadow evaluation and canary comparison before changing tenant defaults.
- Record dataset version, profile version, provider/model version, seed, and
  code revision in every eval run.

Until these gates exist, the product may claim deterministic weighted RRF and
the verified contract behavior in this document, but not universal state of the
art retrieval quality.

### P0: Rerank Contract Hardening

The current rerank SPI accepts only memory ids. A production contract needs the
query, bounded candidate text or references, original ranks and scores, locale,
timeout/cancellation, provider version, and a typed per-candidate result. This
is a public SPI ownership change and requires human review before implementation.

### P1: Optional Dense Retrieval

- Add a qualified embedding provider and a rebuildable vector index behind the
  existing SPI, with explicit model and dimensionality identity.
- Support dual indexing, shadow queries, recall comparison, and rollback during
  model changes.
- Keep canonical records readable and retrievable when the vector provider is
  unavailable.
- Evaluate HNSW/IVF parameters on the actual tenant distribution; do not copy a
  benchmark configuration into production.

Selecting pgvector, OpenSearch, or an external vector service changes database
or provider ownership and requires the corresponding human review.

### P1: Temporal Graph And Semantic Consolidation

- Resolve entity aliases and maintain validity intervals for facts and edges.
- Retrieve bounded graph neighborhoods with temporal predicates and provenance.
- Extend the implemented exact canonical duplicate supersession with qualified
  near-duplicate detection and contradiction policy. Exact-match source
  aggregation and explicit audit/outbox journals are already implemented;
  they are not evidence of semantic duplicate or contradiction resolution.
- Separate event time from ingestion time so late events do not rewrite history.

### P1: Context Optimization

- Replace heuristic token estimates with the selected model tokenizer at the
  provider boundary while keeping deterministic fallback behavior.
- Add query-aware maximal marginal relevance or equivalent diversity selection
  after a qualified embedding/rerank provider is available.
- Measure citation coverage and unsupported-context rate, not only token count.

### P2: Feedback And Adaptive Profiles

- Join explicit feedback and downstream answer support to retrieval traces.
- Learn profile weights offline with tenant and workload segmentation.
- Require monotonic safety constraints, rollback, drift detection, and minimum
  sample sizes before automatic promotion.

## 5. Non-Negotiable Guardrails

- `ai_record` and `ai_event` remain canonical; all search indexes are derived
  and rebuildable.
- Authorization and sensitivity filtering happen before public output and are
  rechecked after provider candidate retrieval.
- Candidate counts remain bounded at the store/provider boundary.
- Capability discovery lists only executable schemes and marks every listed
  scheme with its qualification; evaluation-only families are not advertised
  as selectable production schemes.
- Provider failure degrades explicitly and is recorded in traces; it must not
  silently bypass tenant scope or privacy policy.
- Public API and generated SDK changes follow their authoritative OpenAPI and
  generation path.
- Database migrations, provider lock-in, breaking SPI changes, and generated SDK
  ownership changes require human review.

## 6. Verification

```powershell
cargo test -p sdkwork-memory-retrieval
cargo test -p sdkwork-memory-plugin-reference-profiles
cargo test -p sdkwork-memory-plugin-native-sql
cargo test -p sdkwork-intelligence-memory-service --test retrieval_workflow_contract
node ..\sdkwork-specs\tools\check-component-port-bindings.mjs --root crates\sdkwork-memory-retrieval --strict
```

## 7. Primary References

- Cormack, Clarke, and Buettcher, Reciprocal Rank Fusion Outperforms Condorcet
  and Individual Rank Learning Methods, SIGIR 2009:
  <https://doi.org/10.1145/1571941.1572114>
- Microsoft GraphRAG, graph-based retrieval augmentation:
  <https://github.com/microsoft/graphrag>
- pgvector, exact and approximate vector search for PostgreSQL:
  <https://github.com/pgvector/pgvector>
- Mem0, production-oriented long-term memory evaluation and architecture:
  <https://arxiv.org/abs/2504.19413>
- Zep/Graphiti, temporal knowledge graph memory:
  <https://arxiv.org/abs/2501.13956>

These references define comparison points, not dependency mandates. Provider
selection remains workload- and evaluation-driven.
