# ADR-002: Outbox Exponential Backoff and Dead Letter Queue

- Status: Accepted
- Date: 2026-06-27
- Deciders: Memory Platform Team

## Context

The outbox publisher retried failed events on every poll cycle (every 2 seconds) without any delay based on retry count. This caused excessive load on downstream sinks and the database when events repeatedly failed. Events that exceeded `max_retries` were marked as `failed` but there was no alerting or metric to surface this.

## Decision

1. **Exponential backoff in SQL**: Modify the `claim_global_pending_outbox_events` queries (both PostgreSQL and SQLite) to only claim pending events where `retry_count = 0` or the `updated_at` plus an exponential delay (2^n seconds, capped at 32 seconds) has elapsed. This requires no schema migration.

2. **Dead letter metric**: Add `memory_outbox_dead_letter_total` Prometheus counter incremented when an event transitions to `failed` state. Add a critical Prometheus alert rule (`MemoryOutboxDeadLetter`) that fires when the counter increases within 5 minutes.

## Consequences

- **Positive**: Failed events are retried with exponentially increasing delays, reducing load on downstream systems.
- **Positive**: Dead letter events are immediately surfaced through metrics and alerts, enabling rapid operational response.
- **Positive**: No schema migration required — backoff is computed from existing `retry_count` and `updated_at` columns.
- **Negative**: Events with high retry counts may take up to 32 seconds before the next attempt, slightly delaying recovery after transient failures resolve.
- **Mitigation**: The cap at 32 seconds ensures eventual retry; operators can manually requeue failed events once the root cause is resolved.
