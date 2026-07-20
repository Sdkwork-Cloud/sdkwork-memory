# SDKWork Memory Runtime Topology

SDKWork Memory follows the shared SDKWork runtime topology model.

- Platform standard: `../sdkwork-specs/APP_RUNTIME_TOPOLOGY_SPEC.md`
- Naming authority: `../sdkwork-specs/APP_RUNTIME_TOPOLOGY_NAMING.md`
- Adoption guide: `../sdkwork-specs/APP_RUNTIME_TOPOLOGY_ADOPTION.md`
- Framework: `../sdkwork-app-topology`

## Archetype

The application uses the `application-http-gateway` archetype. The route crates own the Open, App, and Backend API surfaces, while `sdkwork-api-memory-standalone-gateway` assembles the public ingress runtime. Process layout is an internal orchestration detail, not a public profile axis.

## Profiles

The only supported profile IDs are:

| Profile | Purpose | Database |
| --- | --- | --- |
| `standalone.development` | Local development | SQLite by default |
| `standalone.production` | Single-site production | PostgreSQL |
| `cloud.development` | Shared cloud development | Environment-managed |
| `cloud.production` | Kubernetes production | PostgreSQL |

`standalone.development` is the default development profile. `cloud.production` is the default production profile. Source-owned values live in `etc/topology/<deploymentProfile>.<environment>.env`; secrets are injected by the runtime environment and are never committed to profile files.

## Production Deployment

- Container image: `registry.sdkwork.com/apps/sdkwork-memory` from `deployments/docker/Dockerfile`.
- Kubernetes descriptors: `deployments/kubernetes/`, including migration job, Deployment, HPA, PDB, ingress, and Prometheus rules.
- Database: PostgreSQL is required; run migrations through `pnpm db:migrate` or the Kubernetes migration job.
- Drive exports: privacy export jobs upload through SDKWork Drive using deployment-managed credentials.
- Verification: `pnpm verify` and release-readiness checks must pass before publication.

## Local Development

```bash
pnpm dev
pnpm topology:validate
```

| Surface | Default URL |
| --- | --- |
| `application.public-ingress` | `http://127.0.0.1:8080` |
| `application.open-http` | `http://127.0.0.1:8081` |
| `application.app-http` | `http://127.0.0.1:8082` |
| `application.backend-http` | `http://127.0.0.1:8083` |

Browser runtime configuration exposes only declared public keys, including deployment profile and App/Backend/Open SDK base URLs. The PC Console consumes only the App API URL; the PC Admin surface consumes only the Backend API URL.

## Cloud Gateway

Cloud profiles route public traffic through the platform API gateway. Use `pnpm release:package:cloud` for the cloud deployment profile and validate the generated gateway bundle before deployment.
