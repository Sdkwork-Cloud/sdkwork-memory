# Memory Profile Resolver Specs

[`component.spec.json`](component.spec.json) is the machine-readable contract
for resolving implementation profiles and per-port plugin bindings against the
provider-neutral Memory SPI registry.

The native SQL and local embedded phase-1 profiles require both bounded
governance access, atomic user-space quota admission, and bounded scoped
retrieval through `MemoryGovernanceAccessPort`, `MemorySpaceStorePort`, and
`MemoryRetrieverPort`.

The runtime composition boundary is defined in
[`../../../docs/architecture/tech/TECH-2026-06-10-memory-spi-plugin-architecture-design.md`](../../../docs/architecture/tech/TECH-2026-06-10-memory-spi-plugin-architecture-design.md).
Global SDKWork standards remain authoritative through the links in the
component spec.
