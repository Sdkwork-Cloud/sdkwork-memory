# Reference Profiles Plugin Specs

[`component.spec.json`](component.spec.json) is the machine-readable module
contract. The runtime capability and port manifest remains
[`../sdkwork.memory.plugin.json`](../sdkwork.memory.plugin.json).

The plugin is intentionally limited to conformance/evaluation use. See the
canonical SPI design at
`../../../docs/architecture/tech/TECH-2026-06-10-memory-spi-plugin-architecture-design.md`.

Keyword retrieval and context assembly require explicit tenant and space scope
through `retrieve_scoped` and `assemble_scoped`. Their legacy unscoped entrypoints
fail closed in this evaluation runtime.
