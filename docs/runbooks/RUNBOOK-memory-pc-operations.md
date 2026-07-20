# Memory PC Operations Runbook

Status: active

Owner: Memory application operations

Updated: 2026-07-20

## Scope

This runbook covers the browser Console and Admin application. Server incidents continue to use the provider, tenant-isolation, audit, outbox, migration, rate-limit, and key-rotation runbooks in this directory.

## Access Model

- Console users authenticate through the App IAM flow and receive only App permissions.
- Admin operators authenticate through the Backend IAM flow and require explicit internal permissions for every module.
- A missing permission renders a denied state; it does not switch SDK surface or retry through raw HTTP.
- Tenant identity comes from authenticated context. Operators never type a tenant id into a current-tenant command.

Before investigating access, capture the displayed numeric error code and trace id. Never request or paste tokens, cookies, authorization headers, or exported memory content into tickets.

## Runtime Configuration Check

1. Identify the active environment and deployment profile from `runtime-env.json`.
2. Confirm `appApiBaseUrl`, `backendApiBaseUrl`, and `appbaseAppApiBaseUrl` point to the intended environment.
3. Confirm production values are HTTPS and contain no loopback host.
4. Confirm Console calls only `/app/v3/api` and Admin calls only `/backend/v3/api` in browser network diagnostics.
5. Confirm secure response headers and no token values appear in console logs.

Do not edit the built `runtime-env.json` as an emergency fix. Correct the governed `etc/browser` source, rebuild the candidate, and follow release validation.

## Core Smoke Test

1. Sign in to Console with a test tenant/user approved for smoke testing.
2. Open Spaces and request one page of 20 rows.
3. Select a memory and open the update panel; confirm type-aware fields render and the selected version is populated when supported.
4. Open candidates/habits and verify approve/reject commands require their typed reason where configured.
5. Open forget/export history and verify the next-page cursor loads without duplicate or cross-user rows.
6. Sign out, then sign in with an authorized Admin operator.
7. Verify extraction, consolidation, retention, and migration histories load one server page at a time.
8. Verify Provider Health, evaluation, audit, control-plane, graph, and policy modules honor permission denial.
9. Confirm no browser warnings/errors and no horizontal overflow at 390 px and desktop width.

Smoke tests use non-sensitive fixtures or an approved test tenant. Do not create destructive production jobs merely to test the UI.

## Job Investigation

| Job family | Primary surface | Evidence |
| --- | --- | --- |
| Extraction | Admin Learning | `ai_learning_job`, job id, state, result/error, timestamps |
| Consolidation | Admin Learning | typed governance snapshot and supersession audit |
| Forget | Console Governance | actor-constrained history and deletion result |
| Export | Console Governance | actor-constrained history and Drive object reference when used |
| Retention | Admin Governance | typed reason, dry-run flag, deleted-record count |
| Migration | Admin Governance | source/target profile, mode, typed reason, result |

Use the trace id to correlate gateway logs, service spans, and audit records. A UI retry must retain the generated idempotency key for the same logical create/command attempt; a new logical operation receives a new key.

## Failure Response

- `40101`: verify session freshness and IAM issuer/audience configuration; do not bypass authentication.
- `40301`: verify role/permission assignment and tenant/space scope; do not elevate the user as a diagnostic shortcut.
- `40901`: reload the selected resource and reapply the change against its current version.
- `42901`: honor retry metadata and inspect the rate-limit/quota runbook.
- `5xxxx`: capture trace id, stop repeated destructive retries, inspect service health and relevant provider/job runbook.

## Rollback

1. Stop rollout or CDN promotion.
2. Restore the previous immutable browser ZIP or previous host route.
3. Restore the matching public runtime config version.
4. Invalidate only the affected asset paths; preserve immutable hashed assets according to CDN policy.
5. Run Console and Admin smoke tests against the restored version.
6. Record artifact digest, config version, start/end time, reason, and observed recovery signals.

PC rollback does not roll back database migrations or server packages. Coordinate those through the root release and migration runbooks when the incident crosses runtime targets.
