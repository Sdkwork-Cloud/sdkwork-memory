# RUNBOOK: Provider Outage

Status: active  
Owner: SDKWork Memory operators  
Specs: INTEGRATION_SPEC.md, OBSERVABILITY_SPEC.md

## Scope

Respond when optional memory providers (embedding, LLM extraction, Drive export) are unavailable.

## Signals

- `retrieve_provider_health` returns `degraded` or `unhealthy`
- Export jobs fail when `drive_target_ref` is set
- Retrieval continues on keyword/dictionary/time/event/sql paths (embedding optional)

## Mitigation

1. Confirm native SQL retrieval still serves requests (`memory_retrieval_completed_total` increasing).
2. Disable optional provider bindings via backend admin API (`ai_provider_binding` status `disabled`).
3. Route export jobs to inline format until Drive recovers.

## Recovery

1. Restore provider connectivity and set binding health to `healthy`.
2. Re-run provider health check endpoint.
3. Monitor error rates for 30 minutes.

## Escalation

Integration platform on-call → Memory service owner.
