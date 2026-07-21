# Integrator Guide

SDKWork Memory integration for application and platform developers.

## Prerequisites

- SDKWork IAM credentials (auth token + access token for app/backend surfaces, or API key for open surface)
- Generated SDK from `sdks/sdkwork-memory-app-sdk` (app consumers) or `sdks/sdkwork-memory-sdk` (open integrators)
- PostgreSQL for production deployments (SQLite is dev-only)

## Quick start (app API)

1. Obtain auth and access tokens from SDKWork IAM for your tenant.
2. Create a user-owned memory space:

```http
POST /app/v3/api/memory/spaces.create
Authorization: Bearer <auth-token>
Access-Token: <access-token>
Content-Type: application/json

{ "ownerSubjectType": "user", "ownerSubjectId": "<actor-id>", "spaceType": "personal", "displayName": "My memories" }
```

3. Store a memory via event ingestion or direct record create.
4. Retrieve context with `retrievals.create` using a natural-language query.

## Security model

- **Space isolation** — actors may only access spaces they own unless operating with elevated backend context.
- **Sensitivity tiers** — `private`, `sensitive`, and `restricted` records are hidden from list/retrieve/retrieval/export unless the actor owns the space.
- **Fail-closed auth** — production deployments require IAM database resolution; missing credentials return 401/403.

## API surfaces

| Surface | Prefix | Auth | SDK family |
|---------|--------|------|------------|
| Open | `/mem/v3/api` | API key | `sdkwork-memory-sdk` |
| App | `/app/v3/api/memory` | Dual token | `sdkwork-memory-app-sdk` |
| Backend | `/backend/v3/api/memory` | Dual token | `sdkwork-memory-backend-sdk` |

All success responses use `SdkWorkApiResponse`; errors use `application/problem+json` (`ProblemDetail`) with numeric `code` and `traceId`.

## Commercial management (backend)

Backend routes cover subjects, bindings, capability bindings, and capability resolution. See `sdks/sdkwork-memory-backend-sdk/openapi/memory-backend-api.openapi.json` for the authoritative contract.

## Privacy export and Drive

Export jobs that target Drive upload through SDKWork Drive (`sdkwork-memory-drive`). Integrators must not bypass Drive with direct object-store writes. Inline export is bounded to 4 MiB by default; Drive export is bounded to 64 MiB by default and currently uses a single bounded upload buffer. Larger exports must wait for the reviewed streaming multipart SPI rather than raising limits beyond the 256 MiB hard cap.

## List pagination

All list endpoints return `SdkWorkApiResponse` with `data.items` and `data.pageInfo` (`mode: cursor`). Default `page_size` is 20; maximum is 100 unless documented otherwise in OpenAPI.

## Database lifecycle

Production deployments apply schema through the Kubernetes migration Job (`db-migrate`). Runtime pods keep `SDKWORK_MEMORY_DATABASE_AUTO_MIGRATE=false`. The only migration authority is `database/migrations/{postgres,sqlite}`; `database/ddl/baseline/` is generated with `pnpm db:materialize:baseline`. Plugin migration directories contain no SQL authority.

## Further reading

- `docs/product/prd/PRD.md` — product scope and phases
- `docs/architecture/tech/TECH_ARCHITECTURE.md` — runtime topology
- `docs/runbooks/` — operational runbooks
