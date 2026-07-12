# Memory SPI Specs

[`component.spec.json`](component.spec.json) is the machine-readable contract
for the provider- and framework-neutral Memory plugin manifests, registry, and
port traits exported by this crate.

The runtime boundary is defined in
[`../../../docs/architecture/tech/TECH-2026-06-10-memory-spi-plugin-architecture-design.md`](../../../docs/architecture/tech/TECH-2026-06-10-memory-spi-plugin-architecture-design.md).
Global SDKWork standards remain authoritative through the links in the
component spec.

Production retrieval integrations use bounded `search_scoped` with explicit
tenant/space scope, selected retriever kinds, memory-type filters, and a typed
sensitivity read scope. `supports_bounded_scoped_search` is a production
preflight capability probe. The older `retrieve_scoped` method remains for
source compatibility and evaluation helpers; unscoped methods fail closed.
Context assembly uses `assemble_scoped` and also fails closed by default.

Production trace stores expose tenant-scoped trace resolution through
`retrieve_for_tenant` and opt in with `supports_tenant_trace_lookup`. Lookup
must fail closed when a trace identifier is ambiguous across tenant spaces.

Canonical HTTP mutations use the coarse-grained `create_canonical_atomic`,
`update_canonical_atomic`, and `delete_canonical_atomic` methods on
`MemoryRecordStorePort`. Implementations must opt in through
`supports_canonical_atomic`; the default methods fail closed so a plugin cannot
silently split canonical writes from audit and outbox durability.

Governance access uses the coarse-grained `MemoryGovernanceAccessPort`. Its
`resolve_space_governance` method receives explicit tenant/space scope and may
include an actor and one exact capability code. Providers must bound the
returned actor and capability facts to at most `32` rows per category and set
`complete=false` when the bound is exceeded. The service must reject an
incomplete fact set; it must not authorize from a truncated result.

The port returns storage facts only. Tenant, owner, lifecycle, binding role and
validity, capability deny-wins, sensitivity, and request-context policy remain
owned by the Memory service. Unknown or malformed governance values fail
closed. `count_active_records` and `count_user_owned_spaces` remain observation
queries and never act as reservations. Atomic per-space canonical admission is
exposed by `MemoryRecordStorePort::create_canonical_atomic_with_quota`, which
returns a typed admitted-or-quota-exceeded outcome and defaults to fail-closed
for providers that do not implement it. Candidate-to-canonical promotion uses
the same outcome through `MemoryCandidateStorePort::promote_atomic_with_quota`;
its default is also fail-closed. Production promotion uses the additive
`promote_atomic_with_quota_and_journal` method and the
`supports_atomic_candidate_promotion_journal` probe so record, evidence,
candidate state, search projection, outbox, and audit can share one provider
transaction. User-owned space creation is owned by the
separate `MemorySpaceStorePort::create_space_atomic_with_quota` mutation. It
returns a typed admitted-or-quota-exceeded outcome, advertises atomic support
through `supports_atomic_user_space_quota_admission`, and fails closed by
default. Backend supersede uses
`supersede_canonical_atomic_with_quota`: providers serialize quota admission,
link the old and new records, update search projections, and persist both
mutation journals in one atomic boundary. A retry is admitted only when the
existing active target's business fields and both persisted outbox/audit
journals match the command; reused target ids with a different payload or
journal fail with an idempotency conflict. Legacy direct administrative record
writes remain a separate residual boundary.

Promotion workflows obtain candidate evidence, space ownership, timestamps,
and an existing target through the provider-neutral
`MemoryCandidateStorePort::retrieve_detail` projection. The service never
imports a provider SQL row to decide whether a candidate belongs to the
requested space or whether a retry is already promoted. Providers must scope
the lookup by tenant and fail closed when an identifier is ambiguous across
spaces.

HTTP candidate lists use `MemoryCandidateStorePort::list_candidates` with an
explicit tenant, optional space, bounded page size, and opaque cursor. The
provider owns filtering, ordering, and page selection; the service must not
download an unbounded candidate set or consume a provider-specific row type.
`supports_candidate_listing` is required during production preflight and the
default list method fails closed.
