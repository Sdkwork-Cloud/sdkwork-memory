# ADR-002: Fenced Outbox Delivery, Backoff, and Dead Letter State

- Status: Accepted
- Date: 2026-07-21
- Deciders: Memory Platform Team

## Context

Outbox delivery must remain correct when multiple replicas claim work, an HTTP request runs slowly, or a worker stops after delivery but before acknowledgement. Retry timing must also behave identically through the PostgreSQL and SQLite `sqlx::Any` profile.

## Decision

1. Persist `next_attempt_at` and claim only pending rows whose retry schedule is due. Failures use exponential delay from 2 to 32 seconds.
2. Every claim writes `lease_owner`, `lease_token`, and `lease_expires_at`. Renewal, success acknowledgement, and failure recording require the current unexpired owner/token pair.
3. PostgreSQL claims use `FOR UPDATE SKIP LOCKED`; SQLite claims use one write transaction and conditional updates.
4. The publisher uses bounded concurrency, renews leases while delivery is in flight, disables redirects, and uses the shared SSRF-protected pinned HTTP client.
5. Exhausted retries enter terminal `failed` state and increment `memory_outbox_dead_letter_total`.

## Consequences

- Expired workers cannot overwrite the result of a replacement worker.
- Retry and expiry behavior is observable from stored timestamps rather than inferred from `updated_at`.
- Consumers must remain idempotent because the transactional outbox provides at-least-once delivery, not impossible-to-prove exactly-once delivery.
- Lease and retry columns require reviewed forward and down migrations on both engines.
