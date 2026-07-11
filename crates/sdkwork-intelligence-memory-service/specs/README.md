# SDKWork Memory Service Specs

`component.spec.json` is the machine-readable integration contract for the Memory L2 service.

The public `MemoryRuntimeDataPlane` facade dispatches service operations through the typed ports in
`sdkwork-memory-spi`. Production HTTP composition requires every Phase-1 store port at startup.
Evaluation profiles may exercise declared optional ports, but they do not qualify for production
HTTP cutover without the complete service, governance, privacy, and transactional contracts.

Canonical standards remain in `../../../sdkwork-specs/`.
