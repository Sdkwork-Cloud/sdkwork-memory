# ADR-003: API Rate Limiting and Body Size Limits

- Status: Accepted
- Date: 2026-06-27
- Deciders: Memory Platform Team

## Context

The API server had no request body size limits or concurrency controls. A single large request or a burst of concurrent requests could exhaust server resources (memory, database connections), causing denial of service for all tenants.

## Decision

Add two configurable middleware layers to the final router in `bootstrap.rs`:

1. **`DefaultBodyLimit`**: Limits request body size to 1 MiB by default, configurable via `SDKWORK_MEMORY_MAX_BODY_BYTES`.

2. **`ConcurrencyLimitLayer`**: Limits concurrent in-flight requests to 256 by default, configurable via `SDKWORK_MEMORY_MAX_CONCURRENCY`. Excess requests receive HTTP 503.

## Consequences

- **Positive**: Protects against memory exhaustion from large payloads and connection pool exhaustion from request floods.
- **Positive**: Both limits are per-pod and configurable at deploy time through environment variables.
- **Negative**: These are per-instance limits, not distributed rate limits. In a multi-pod deployment, the effective limit is `N × max_concurrency`.
- **Future**: Distributed rate limiting (e.g., Redis-based token bucket per tenant) should be added for precise per-tenant throttling across replicas.
