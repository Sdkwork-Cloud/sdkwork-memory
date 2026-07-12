# Reference Profiles Plugin Specs

[`component.spec.json`](component.spec.json) is the machine-readable module
contract. The runtime capability and port manifest remains
[`../sdkwork.memory.plugin.json`](../sdkwork.memory.plugin.json).

The plugin is intentionally limited to conformance/evaluation use. See the
canonical SPI design at
`../../../docs/architecture/tech/TECH-2026-06-10-memory-spi-plugin-architecture-design.md`.

Keyword/dictionary/time candidate search and context assembly require explicit
tenant and space scope through bounded `search_scoped` and `assemble_scoped`.
The legacy `retrieve_scoped` helper remains for compatibility. Unscoped
entrypoints fail closed in this evaluation runtime.

The evaluation trace store supports tenant-scoped `retrieve_for_tenant` lookup
and rejects trace identifiers that are ambiguous across spaces instead of
selecting an arbitrary trace.

The reference runtime declares `MemoryGovernanceAccessPort` only for bounded
conformance and evaluation. It follows the same `32`-fact ceiling and
`complete=false` fail-closed contract, while authorization and malformed-state
policy remain owned by the Memory service. This declaration does not qualify
the plugin for production HTTP selection.

Reference quota counts are evaluation observations, not reservations. Its
canonical record port also implements the typed per-space quota admission
contract under the runtime's records lock, with rollback-safe outbox/audit
state. Atomic supersede applies the retained-record quota, both chain links,
and both mutation journals under the same in-memory mutation boundary. Retry
acceptance requires the existing target business projection and both immutable
journal payloads to match the command; collisions fail closed.
Candidate promotion implements the typed promotion port with retry
idempotency and the same records-lock admission semantics. This remains an
evaluation model: SQL evidence rows are not materialized, while the additive
journal-aware promotion method atomically updates its in-memory outbox and
audit projections for conformance. `MemorySpaceStorePort` provides the same typed user-space quota
admission under the governance-space mutex for concurrent conformance tests,
without claiming durable SQL locking or production qualification.

Candidate detail lookup is provider-neutral and tenant-scoped. The reference
runtime preserves creation and decision timestamps, resolves the optional
promotion target, and returns an ambiguity error when the same candidate id is
present in multiple spaces for one tenant, matching the fail-closed production
contract.
The reference list implementation maintains tenant and tenant/space ordered
indexes plus a write-time tenant ambiguity index as candidates are written,
then walks at most one requested page plus look-ahead. It never rebuilds or
sorts the full candidate set during a read. Unscoped lists fail closed before
returning any page when duplicate candidate ids would make a later cursor
ambiguous across spaces.
