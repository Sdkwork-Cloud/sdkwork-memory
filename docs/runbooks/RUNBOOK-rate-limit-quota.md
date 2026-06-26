# RUNBOOK: Rate Limit and Quota Incidents

Status: active  
Owner: SDKWork Memory operators  
Specs: SECURITY_SPEC.md, WEB_FRAMEWORK_SPEC.md

## Scope

Handle abuse or quota exhaustion on memory APIs.

## Signals

- HTTP 429 responses from web framework rate limit interceptor
- HTTP 429 responses with `quota_exceeded` problem code from tenant/space quotas
- Prometheus counter `memory_quota_exceeded_total` increasing
- Auth-critical operation throttling (forget/export/delete)
- Tenant complaint of blocked legitimate traffic

## Investigation

1. Inspect HTTP metrics: `sdkwork_http_requests_labeled_total` with elevated 429 status.
2. Inspect domain metrics: `memory_quota_exceeded_total` and `memory_authz_denied_total`.
3. Review `SDKWORK_MEMORY_MAX_RECORDS_PER_SPACE` and `SDKWORK_MEMORY_MAX_SPACES_PER_USER` (0 = unlimited).
4. Identify hot `operationId` and tenant from structured logs (no raw tokens).
5. Review route manifest rate limit tiers in `crates/sdkwork-routes-memory-*/src/http_route_manifest.rs`.

## Mitigation

- Temporary tenant block at gateway/IAM layer
- Increase limit tier only with security approval
- Enable idempotency keys for retried mutations

## Escalation

Platform SRE → Memory service owner → IAM for tenant policy updates.
