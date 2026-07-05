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

Backend routes cover subjects, bindings, capability bindings, and capability resolution. See `memory-backend-api.openapi.json` for the authoritative contract.

## Database lifecycle

Production deployments apply schema through the Kubernetes migration Job (`db-migrate`). Runtime pods keep `SDKWORK_MEMORY_DATABASE_AUTO_MIGRATE=false`. Canonical DDL lives in `database/ddl/baseline/`; plugin migrations in `plugins/sdkwork-memory-plugin-native-sql/migrations/` are folded into baseline during materialization.

## Further reading

- `docs/product/prd/PRD.md` — product scope and phases
- `docs/architecture/tech/TECH_ARCHITECTURE.md` — runtime topology
- `docs/runbooks/` — operational runbooks
