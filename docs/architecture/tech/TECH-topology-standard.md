> Migrated from `docs/topology-standard.md` on 2026-06-24.
> Owner: SDKWork maintainers

This repository adopts the shared SDKWork runtime topology framework.

- Platform standard: `../sdkwork-specs/APP_RUNTIME_TOPOLOGY_SPEC.md`
- Naming authority: `../sdkwork-specs/APP_RUNTIME_TOPOLOGY_NAMING.md`
- Adoption guide: `../sdkwork-specs/APP_RUNTIME_TOPOLOGY_ADOPTION.md`
- Framework: `../sdkwork-app-topology`

## Archetype

`application-http-gateway`: Memory exposes open, app, and backend HTTP surfaces through `sdkwork-routes-memory-*` route crates. Phase 1 runs all surfaces in a unified `sdkwork-memory-standalone-gateway` process.

## Default Dev Profile

`standalone.unified-process.development`

```bash
pnpm dev
pnpm topology:validate
```

## Local URLs

| Surface | URL |
| --- | --- |
| `application.public-ingress` | http://127.0.0.1:8080 |
| `application.app-http` | http://127.0.0.1:8080 |
| `application.backend-http` | http://127.0.0.1:8080 |
| `application.open-http` | http://127.0.0.1:8080 |

Client env keys:

- `VITE_SDKWORK_MEMORY_DEPLOYMENT_PROFILE`: browser-visible deployment profile.
- `VITE_SDKWORK_MEMORY_APPLICATION_PUBLIC_HTTP_URL`: unified ingress surface.
- `VITE_SDKWORK_MEMORY_APPLICATION_APP_HTTP_URL`: app SDK surface.
- `VITE_SDKWORK_MEMORY_APPLICATION_BACKEND_HTTP_URL`: backend SDK surface.
- `VITE_SDKWORK_MEMORY_APPLICATION_OPEN_HTTP_URL`: open SDK surface.

Profile values live in `configs/topology/*.env` only.

## Phase 1 Notes

Cloud split-services profiles and API gateway bundles are deferred until Memory moves to split-service deployment. Unified-process profiles bind all surfaces to the same host/port.

