# SDKWork Memory Component Specs

This directory is the local component contract entrypoint for `sdkwork-memory`.

Authoritative root standards remain in `../sdkwork-specs/`. Local specs may narrow or instantiate those standards for Memory, but they must not redefine them.

Primary standards for this component:

- `../sdkwork-specs/SOUL.md`
- `../sdkwork-specs/COMPONENT_SPEC.md`
- `../sdkwork-specs/NAMING_SPEC.md`
- `../sdkwork-specs/DOMAIN_SPEC.md`
- `../sdkwork-specs/API_SPEC.md`
- `../sdkwork-specs/SDK_SPEC.md`
- `../sdkwork-specs/SDK_WORKSPACE_GENERATION_SPEC.md`
- `../sdkwork-specs/WEB_BACKEND_SPEC.md`
- `../sdkwork-specs/DATABASE_SPEC.md`
- `../sdkwork-specs/EVENT_SPEC.md`
- `../sdkwork-specs/PRIVACY_SPEC.md`
- `../sdkwork-specs/OBSERVABILITY_SPEC.md`

Local design authority:

- `docs/architecture/tech/TECH-2026-06-10-ai-memory-architecture-design.md`
- `docs/architecture/tech/TECH-2026-06-10-memory-spi-plugin-architecture-design.md`

Legacy redirect stubs (do not edit content here):

- `docs/superpowers/specs/2026-06-10-ai-memory-architecture-design.md`
- `docs/superpowers/specs/2026-06-10-memory-spi-plugin-architecture-design.md`

Draft contract artifacts:

- `docs/schema-registry/tables/*.yaml`
- `sdks/sdkwork-memory-sdk/openapi/memory-open-api.openapi.json`
- `sdks/sdkwork-memory-app-sdk/openapi/memory-app-api.openapi.json`
- `sdks/sdkwork-memory-backend-sdk/openapi/memory-backend-api.openapi.json`
- `sdks/sdkwork-memory-sdk/sdk-manifest.json`
- `sdks/sdkwork-memory-app-sdk/sdk-manifest.json`
- `sdks/sdkwork-memory-backend-sdk/sdk-manifest.json`

Phase 1 verification:

```powershell
node tools/materialize_phase1_contracts.mjs
powershell -ExecutionPolicy Bypass -File tools/verify_phase1.ps1
```
