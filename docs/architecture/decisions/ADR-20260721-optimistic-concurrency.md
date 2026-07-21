# ADR-20260721: Optimistic Concurrency for Versioned Memory Resources

Status: proposed, not implemented

Owner: SDKWork Memory maintainers

Date: 2026-07-21

Human review: required before implementation because this changes public API behavior

Specs: `API_SPEC.md`, `SDK_SPEC.md`, `DATABASE_SPEC.md`, `TEST_SPEC.md`, `ARCHITECTURE_DECISION_SPEC.md`

## Context

Memory resources expose a `version`, but current update and soft-delete SQL increments the stored version without checking the version observed by the caller. Two writers can therefore read version 1, both update successfully, and silently overwrite one another. This is incompatible with commercial administration, multi-replica deployment, and high-concurrency correctness.

The affected surface includes versioned mutable resources on App, Backend, and Open APIs, including spaces, memories, habits, subjects, bindings, capability bindings, entities, edges, policies, policy assignments, implementation profiles, retrieval profiles, provider bindings, indexes, and other admin resources that expose `version`.

No public concurrency field or `If-Match` header is currently authoritative. Implementing one without review would be a breaking API change. This record therefore defines the proposed contract and remains non-authoritative until accepted.

## Decision

Use the existing entity `version` as the only concurrency token and HTTP `If-Match` as the mutation precondition.

- A strong entity tag is the quoted decimal version, for example `ETag: "7"`.
- Versioned single-resource reads, creates, and successful updates return `ETag` for the returned representation.
- Versioned `PATCH`, `PUT`, and `DELETE` operations accept exactly one strong `If-Match` value. Weak tags, wildcard `*`, multiple tags, non-decimal values, zero, negative values, and values outside the persisted integer range are invalid.
- Once enforcement is enabled, a missing required header returns HTTP 428 with code `42801 PRECONDITION_REQUIRED`.
- A well-formed header that does not match the current resource version returns HTTP 412 with code `41201 PRECONDITION_FAILED`.
- `40901 CONFLICT` remains reserved for unique-key, lifecycle-state, and other domain conflicts that are not direct HTTP precondition failures.
- Create operations do not accept `If-Match`.
- A repeated mutation may replay through the existing idempotency store when the same idempotency key and fingerprint are used. Without such a replay, a delete against a resource that no longer has a live representation returns the existing not-found response.

Tenant and authorization predicates must be evaluated together with the version predicate. A caller must never learn the current version of a resource outside its authorized tenant, space, or actor scope.

## API and SDK Ownership

The change must start in authored OpenAPI and route authority, then be materialized and regenerated. Generated transports must not be hand-edited.

For each affected operation:

1. Add the canonical `If-Match` request header parameter to authored OpenAPI.
2. Declare `ETag` response headers on single-resource success responses.
3. Declare 400, 404, 412, and 428 Problem Details responses as applicable.
4. Materialize authority OpenAPI and route manifests.
5. Regenerate Open, App, and Backend SDK families.
6. Update composed SDK facades only where a stable version-to-ETag helper is needed.
7. Update PC services to pass the version observed in their SDK DTO through the generated `If-Match` argument. No raw HTTP or manual authorization headers are allowed.

The wire representation is deliberately derived from the existing version. List screens may therefore create the strong tag from an item version without an extra retrieve request. The helper must reject invalid or lossy numeric conversions; JavaScript consumers must preserve SDKWork `int64` serialization rather than coercing large versions through `number`.

## Database and Transaction Semantics

No schema migration is proposed because the affected tables already carry `version`.

Every versioned mutation must add the expected version to the same atomic statement as tenant, resource, live-state, and other authorization predicates:

```sql
UPDATE ai_entity
SET canonical_name = ?, updated_at = ?, version = version + 1
WHERE tenant_id = ?
  AND uuid = ?
  AND status <> 'deleted'
  AND version = ?;
```

The same rule applies to soft delete. Successful updates return or reload the incremented version. Zero affected rows are classified without weakening tenant isolation:

- no authorized live resource: existing not-found response;
- authorized live resource with a different version: `41201`;
- authorized resource with matching version but invalid lifecycle state: `40901` or an approved domain code.

Business mutation, Outbox, and Audit remain in one transaction. The business row must be updated first. If its version predicate affects zero rows, the transaction must end without inserting Outbox, Audit, search-index, or other derived side effects.

SQLite and PostgreSQL must use equivalent predicates and error classification. PostgreSQL row locks may support multi-step operations, but row locking must not replace the version predicate. SQLite transactions must retain bounded lock duration and must not use read-then-write comparison outside the mutation statement.

## Compatibility and Rollout

Two rollout paths are acceptable after human review:

- Pre-launch enforcement: because Memory is still a release candidate, update all first-party SDK and PC consumers in one governed breaking release, then require `If-Match` immediately.
- Compatibility window: first emit ETags and accept an optional header while measuring legacy calls; update all approved consumers; then make the header required in the next declared breaking API version. The application must not claim lost-update protection during the optional phase.

The chosen path must be recorded before implementation. Silent indefinite optional behavior is not acceptable.

## Alternatives

### Required `expectedVersion` body field

This is easy for generated SDKs but duplicates HTTP conditional semantics, cannot naturally cover standard bodyless DELETE operations, and creates another version vocabulary. It is not selected.

### Database locks without a caller precondition

Locks serialize individual statements but cannot detect that a user is writing from a stale representation. They do not prevent lost intent and are not selected.

### Last-write-wins

This is the current effective behavior. It is unsuitable for policy, binding, graph, profile, and administrative resources where silent overwrites create security and operational risk.

## Consequences

Positive consequences:

- Concurrent writes become deterministic across replicas and database dialects.
- Stale mutations cannot create business, Outbox, and Audit divergence.
- The existing version field gains one clear cross-layer meaning.
- HTTP, OpenAPI, generated SDKs, UI, and SQL use one traceable precondition contract.

Costs and risks:

- Enforcing the header breaks clients that mutate without a concurrency token.
- Every versioned update and delete path must be audited; partial adoption would provide a false safety claim.
- SDK and UI error handling must support 412 and 428 explicitly.
- Conflict classification requires an additional scoped existence/version query when an update affects zero rows, unless an equivalent dialect-safe statement returns enough information.

## Verification

Implementation is not complete until all of the following pass:

- OpenAPI/route/SDK materialization and ownership checks for all three surfaces.
- Static checks proving all versioned mutations bind expected version in SQL.
- SQLite and real PostgreSQL tests with two writers that read the same version: one succeeds, one returns 412, and the winner's data remains intact.
- Transaction tests proving a stale business mutation creates no Outbox, Audit, search-index, or derived rows.
- Missing, weak, wildcard, malformed, overflow, cross-tenant, cross-space, and deleted-resource negative tests.
- Generated SDK tests proving `If-Match` is emitted without raw HTTP wrappers.
- PC service tests proving stale responses trigger reload/reconciliation rather than automatic mutation replay.
- Contention tests across multiple server replicas and PostgreSQL connections.
- `cargo clippy --workspace --all-targets -- -D warnings`, `pnpm check`, and `pnpm verify`.

## Implementation State

Not implemented. Current SQL remains last-write-wins. Acceptance of this ADR and selection of a rollout path are required before public contract and persistence changes begin.

## Supersedes / Superseded By

None.
