# Native SQL Plugin Specs

[`component.spec.json`](component.spec.json) is the machine-readable module
contract. The runtime capability and port manifest remains
[`../sdkwork.memory.plugin.json`](../sdkwork.memory.plugin.json).

The plugin follows the canonical SPI design at
`../../../docs/architecture/tech/TECH-2026-06-10-memory-spi-plugin-architecture-design.md`
and the global SDKWork standards linked from `component.spec.json`.

Its Phase-1 retriever implements bounded `search_scoped` for SQL, keyword,
dictionary, time, and linked-event candidates. Memory type and sensitivity
filters are applied before SQL `LIMIT`; the service rehydrates canonical
records before returning hits. SQLite/PostgreSQL full-text failures fall back
only when the FTS capability is recognizably unavailable; unrelated database
failures are propagated instead of being mislabeled as degraded retrieval.

Its retrieval trace store advertises `supports_tenant_trace_lookup` and resolves
trace identifiers through tenant-scoped `retrieve_for_tenant` without an
unscoped fallback. Trace, hit, and optional context-pack rows are appended in a
single transaction.

Its governance adapter advertises `supports_bounded_governance_access` and
resolves one tenant-scoped space together with actor-binding facts and facts for
one exact capability code. Tenant, space, actor, target, and capability filters
are applied before SQL `LIMIT`. Each fact category is capped at `32`; overflow
sets `complete=false`, which the service must treat as a fail-closed result.

The adapter does not decide authorization. Owner, lifecycle, role, validity,
capability precedence, sensitivity, and malformed-state handling remain in the
Memory service. The two quota count methods return non-negative observations;
they do not reserve quota. Canonical record creation and Native SQL candidate
promotion instead use the record-space row as a transaction lock, count active
records before writing, and keep record side effects plus SQLite FTS in that
transaction. Candidate promotion journal durability is no longer a residual:
the journal-aware promotion port appends outbox and audit rows before the same
commit. User-owned space creation uses
`MemorySpaceStorePort` to lock the stable
`ops_memory_schema_version` `0001` row, count active owner spaces, and insert in
one transaction. PostgreSQL uses `FOR UPDATE`; SQLite uses a no-op update as its
first write. This is cross-instance safe without a migration but deliberately
serializes all space creation. A future per-owner quota ledger requires a
reviewed schema migration. Backend supersede also uses the space lock: quota
admission, the new record, both chain links, two outbox/audit journals, SQLite
FTS insertion/removal, and commit are one transaction. Legacy direct
administrative record helpers remain explicit residual paths. Supersede retry
acceptance revalidates the target business projection and both immutable
journal payloads inside the same serialization transaction; a reused target id
with different data fails as an idempotency conflict.

The candidate adapter also maps its tenant-scoped SQL detail query into the
provider-neutral `MemoryCandidateDetail` projection. Evidence JSON, space id,
timestamps, and the target memory id are all returned through the SPI; the
service does not depend on `NativeSqlCandidateDetailRow`.
Candidate lists are filtered and cursor-paged in SQL before projection into
`MemoryCandidateSummary`, preserving the existing stable UUID ordering without
exposing `NativeSqlCandidateRow` to service code.
