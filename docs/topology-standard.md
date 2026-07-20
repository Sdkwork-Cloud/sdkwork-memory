# SDKWork Memory Runtime Topology

This repository adopts the shared SDKWork runtime topology framework.

- Platform standard: `../sdkwork-specs/APP_RUNTIME_TOPOLOGY_SPEC.md`
- Naming authority: `../sdkwork-specs/APP_RUNTIME_TOPOLOGY_NAMING.md`
- Adoption guide: `../sdkwork-specs/APP_RUNTIME_TOPOLOGY_ADOPTION.md`
- Framework: `../sdkwork-app-topology`

## Archetype

`application-http-gateway`: Memory exposes open, app, and backend HTTP surfaces through `sdkwork-routes-memory-*` route crates. The default production profile (`cloud.split-services.production`) runs `sdkwork-api-memory-standalone-gateway` as a unified ingress process; split-service profiles are available when surfaces must bind to separate hosts.

## Production deployment

- **Container image:** `registry.sdkwork.com/apps/sdkwork-memory` (see `deployments/docker/Dockerfile`)
- **Kubernetes:** `deployments/kubernetes/` — migration Job, Deployment, HPA, PDB, Prometheus rules
- **Database:** PostgreSQL required; apply migrations via `pnpm db:migrate` locally or the K8s migration Job in production
- **Drive exports:** privacy export jobs upload through SDKWork Drive (`sdkwork-memory-drive`); configure `sdkwork-memory-drive` secrets per `deployments/kubernetes/secret.example.yaml`
- **Verification:** `pnpm verify` before release

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

## Cloud profiles

`cloud.split-services.production` is the default production profile in `specs/topology.spec.json`. Use `pnpm gateway:validate:cloud` and `pnpm gateway:package:cloud` when packaging cloud gateway bundles.
