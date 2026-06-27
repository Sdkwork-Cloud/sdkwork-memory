# ADR-004: Commercial Memory Management Architecture

- Status: Accepted
- Date: 2026-06-27
- Deciders: Memory Platform Team

## Context

The Memory service lacked a commercial management layer for binding memory spaces to subjects (tenants, organizations, users, applications), assigning capabilities, and enforcing policies. This prevented the system from being used in multi-tenant commercial deployments where different customers have different access levels and feature entitlements.

The database schema (migration 0007) already defined tables for `ai_subject`, `ai_memory_binding`, `ai_capability_binding`, and `ai_policy_assignment`, but there was no service-layer implementation or API surface.

## Decision

Implement a three-layer commercial management architecture:

1. **Contract layer** (`sdkwork-memory-contract/src/commercial.rs`): Define DTOs for `MemorySubject`, `MemoryBinding`, `MemoryCapabilityBinding`, and `ResolvedCapability` with serde serialization matching the OpenAPI camelCase convention.

2. **Store layer** (`sdkwork-memory-plugin-native-sql/src/commercial_store.rs`): Implement CRUD methods on `NativeSqlMemoryStore` for subjects, bindings, and capability bindings. All methods use the existing `AnyPool` and support both PostgreSQL and SQLite. Soft deletes (`deleted_at`) are used for audit trails.

3. **Service layer** (`sdkwork-intelligence-memory-service/src/commercial_api.rs`): Implement business logic on `OpenMemoryService` including Snowflake ID generation, tenant validation, and DTO-to-row mapping. The `resolve_capabilities` method provides capability resolution for a given target.

4. **API layer** (`sdkwork-routes-memory-backend-api/src/commercial_routes.rs`): Expose 15 REST endpoints under `/backend/v3/api/memory/` for subject, binding, and capability binding management. All endpoints require backend authentication and enforce tenant ID matching.

## Consequences

- **Positive**: Full CRUD for commercial entities enables multi-tenant billing, access control, and capability management.
- **Positive**: Capability resolution is integrated into the retrieval and write pipelines. The `access` module resolves `memory.retrieve` and `memory.write` capabilities for each space before allowing operations. Deny wins over allow per design §8.2. Backend operators with elevated tenant access bypass capability checks for administrative operations.
- **Positive**: Soft deletes preserve audit trails for compliance.
- **Positive**: Capability validity periods (`valid_from`/`valid_to`) are enforced at resolution time using ISO 8601 lexicographic comparison.
