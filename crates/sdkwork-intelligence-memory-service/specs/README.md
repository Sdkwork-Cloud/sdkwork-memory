# SDKWork Memory Service Specs

`component.spec.json` is the machine-readable integration contract for the Memory L2 service.

The public `MemoryRuntimeDataPlane` facade dispatches service operations through the typed ports in
`sdkwork-memory-spi`. Production HTTP composition requires every Phase-1 store port and a bounded,
scope-aware `MemoryRetrieverPort` at startup. The trace store must also support tenant-scoped
`retrieve_for_tenant` lookup so trace retrieval cannot fall back to an unscoped identifier search.
Production composition also requires `MemoryGovernanceAccessPort` and its
`supports_bounded_governance_access` probe. Governance resolution is capped at `32` actor facts and
`32` exact-capability facts for one tenant-scoped space. A provider reports overflow through
`complete=false`; the service rejects that result instead of authorizing from truncated evidence.
Retrieval candidates are never canonical truth: the service rehydrates every candidate through
`MemoryRecordStorePort`, then rechecks scope, sensitivity, deletion, and request filters before
fusion and trace persistence. Feedback lookup and retrieval-quality evaluation use the same typed
trace/retriever ports. Reading an existing multi-space trace filters hits that are no longer
authorized and recomputes the caller-visible ranks and result count.

Governance adapters return facts, while this service owns elevated access, ownership, lifecycle,
binding role and validity, capability precedence, sensitivity, and malformed-state policy. Unknown
or invalid governance data fails closed. Read/retrieval/write authorization resolves lifecycle,
actor access, owner status, and the exact operation capability from one bounded governance snapshot
per space. Multi-space workflows deduplicate repeated space ids without changing first-seen order,
and context-pack creation delegates to the retrieval workflow instead of authorizing the same spaces
twice. Governance quota counts remain non-negative observations for diagnostics and management only.
Canonical create now passes a typed per-space quota policy into the record-store mutation, so the
space lock, active-record count, record, journal, and SQLite search projection share one transaction.
Candidate promotion dispatches through `MemoryCandidateStorePort`; Native SQL uses the same space
admission boundary and re-reads its candidate target inside that transaction for retry idempotency,
while `promote_atomic_with_quota_and_journal` commits outbox and audit with the record, evidence,
target, approval, and search projection. The reference runtime provides the same typed journal behavior
for evaluation. User-owned space creation
now dispatches through `MemorySpaceStorePort`: the provider serializes admission, counts active spaces,
and inserts the new space in one atomic boundary. The service consumes the typed quota outcome and maps
rejection to the existing HTTP 429 contract. Backend supersede dispatches
through the atomic record-store port and
commits quota admission, both chain links, old/new journals, and search-index
changes together. Only legacy direct administrative fixture/write helpers
remain outside this HTTP mutation boundary.
Promotion and backend approval first consume the provider-neutral
`retrieve_detail` projection through `MemoryRuntimeDataPlane`; provider SQL row
types are confined to the adapter and cannot leak into service orchestration.
The detail lookup is tenant-scoped and must reject ambiguous candidate ids
instead of selecting an arbitrary space.
Candidate list and retrieve HTTP paths use the same typed runtime boundary.
Lists are cursor-paged by the provider and return provider-neutral summaries;
the service contains no Native SQL candidate row imports or direct SQL store
list/look-up calls.
Evaluation profiles may exercise declared optional ports, but they do not qualify for production
HTTP cutover without the complete service, governance, privacy, and transactional contracts.

Canonical standards remain in `../../../sdkwork-specs/`.
