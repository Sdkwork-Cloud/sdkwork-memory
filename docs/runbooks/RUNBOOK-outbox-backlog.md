# RUNBOOK: Outbox Backlog and Delivery Failures

Status: active  
Owner: SDKWork Memory operators  
Specs: `EVENT_SPEC.md`, `DATABASE_SPEC.md`

## Scope

Handle growing outbox backlog, expired delivery leases, or repeated delivery failures for Memory domain events.

## Architecture

1. Mutations append `pending` rows in the same transaction as domain state.
2. The publisher claims bounded batches and writes `lease_owner`, `lease_token`, and `lease_expires_at` while moving rows to `processing`.
3. HTTP delivery emits a CloudEvents-style envelope and renews the lease while the request is in flight.
4. Success and failure updates require the current unexpired token. Failed attempts return to `pending` with `next_attempt_at`, or enter terminal `failed` state.
5. Expired processing leases are requeued. An old worker cannot acknowledge after another worker takes ownership.

## Signals

- `memory_outbox_pending_total` rising for an extended period
- `memory_outbox_delivery_failed_total` increasing
- `memory_outbox_publish_failed_total` non-zero
- `memory_outbox_dead_letter_total` increasing
- `processing` rows whose `lease_expires_at` is in the past

## Investigation

1. Confirm the publisher is running in the standalone gateway replicas.
2. Production requires `SDKWORK_MEMORY_OUTBOX_DELIVERY_MODE=http`; disabled mode intentionally leaves rows pending.
3. Verify `SDKWORK_MEMORY_OUTBOX_DELIVERY_URL`, TLS, DNS answers, and egress policy without recording the URL or payload in tickets.
4. Query state and lease age:

```sql
SELECT publish_state, COUNT(*)
FROM ai_outbox_event
GROUP BY publish_state
ORDER BY publish_state;

SELECT tenant_id, uuid, event_type, retry_count, next_attempt_at,
       lease_owner, lease_expires_at, created_at
FROM ai_outbox_event
WHERE publish_state IN ('pending', 'processing', 'failed')
ORDER BY created_at ASC
LIMIT 20;
```

## Mitigation

- For transient downstream errors, allow scheduled retry and confirm `SDKWORK_MEMORY_OUTBOX_MAX_RETRIES`.
- For expired leases, verify database time and worker health; normal requeue should recover without manual updates.
- For terminal failures, fix the consumer and use an approved, audited replay procedure. Never edit payloads in place.
- Scale publisher concurrency only after checking database capacity and downstream rate limits.

## Escalation

Platform SRE -> Memory service owner -> downstream event consumer owner.
