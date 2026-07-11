# Memory SPI Specs

[`component.spec.json`](component.spec.json) is the machine-readable contract
for the provider- and framework-neutral Memory plugin manifests, registry, and
port traits exported by this crate.

The runtime boundary is defined in
[`../../../docs/architecture/tech/TECH-2026-06-10-memory-spi-plugin-architecture-design.md`](../../../docs/architecture/tech/TECH-2026-06-10-memory-spi-plugin-architecture-design.md).
Global SDKWork standards remain authoritative through the links in the
component spec.

Retriever and context-assembler integrations should use `retrieve_scoped` and
`assemble_scoped`. Both methods default to fail-closed until a plugin provides
an implementation; the legacy unscoped methods remain only for source
compatibility.

Canonical HTTP mutations use the coarse-grained `create_canonical_atomic`,
`update_canonical_atomic`, and `delete_canonical_atomic` methods on
`MemoryRecordStorePort`. Implementations must opt in through
`supports_canonical_atomic`; the default methods fail closed so a plugin cannot
silently split canonical writes from audit and outbox durability.
