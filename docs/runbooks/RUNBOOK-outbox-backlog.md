# RUNBOOK: Outbox Backlog and Delivery Failures

Status: active  
Owner: SDKWork Memory operators  
Specs: EVENT_SPEC.md, DATABASE_SPEC.md

## Scope

Handle growing outbox backlog, stuck `processing` rows, or repeated delivery failures for domain events emitted by Memory mutations.

## Architecture

1. Mutations append rows to `ai_outbox_event` with `publish_state = pending`.
2. The outbox publisher claims batches with `FOR UPDATE SKIP LOCKED`, moving rows to `processing`.
3. The delivery adapter posts CloudEvents-style envelopes (log or HTTP webhook).
4. Successful delivery calls `ack_outbox_delivery_success` (`processing` → `published`).
5. Failed delivery calls `record_outbox_delivery_failure` (retry to `pending` or terminal `failed`).
6. Stale `processing` rows are requeued by `requeue_stale_processing_outbox_events`.

## Signals

- `memory_outbox_pending_total` rising for extended periods
- `memory_outbox_delivery_failed_total` increasing
- `memory_outbox_publish_failed_total` (claim failures) non-zero
- Rows stuck in `processing` beyond `SDKWORK_MEMORY_OUTBOX_STALE_PROCESSING_SECS` (default 300)

## Investigation

1. Confirm outbox publisher is running (embedded in api-server bootstrap or dedicated worker).
2. Inspect delivery mode: `SDKWORK_MEMORY_OUTBOX_DELIVERY_MODE` (`log` or `http`).
3. For HTTP mode, verify `SDKWORK_MEMORY_OUTBOX_DELIVERY_URL` reachability and TLS.
4. Query backlog by state:

```sql
SELECT publish_state, COUNT(*)
FROM ai_outbox_event
GROUP BY publish_state
ORDER BY publish_state;
```

5. Sample oldest pending rows (tenant-scoped, no payload secrets in tickets):

```sql
SELECT tenant_id, uuid, event_type, retry_count, created_at
FROM ai_outbox_event
WHERE publish_state IN ('pending', 'processing', 'failed')
ORDER BY created_at ASC
LIMIT 20;
```

## Mitigation

- **Transient webhook errors**: wait for automatic retry; confirm `SDKWORK_MEMORY_OUTBOX_MAX_RETRIES` (default 5).
- **Stuck processing**: restart publisher or wait for stale requeue; verify no long-running DB transactions holding locks.
- **Terminal failed rows**: fix downstream consumer, then manually requeue or replay from audit if policy allows.
- **Claim failures**: check DB connectivity and replica lag; scale publisher only after confirming SKIP LOCKED claim path is healthy.

## Escalation

Platform SRE → Memory service owner → downstream event consumer owner.
