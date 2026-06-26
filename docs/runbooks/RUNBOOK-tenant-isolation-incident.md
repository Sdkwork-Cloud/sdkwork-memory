# RUNBOOK: Tenant Isolation Incident Response

Status: active  
Owner: SDKWork Memory security  
Application: sdkwork-memory  
Specs: SECURITY_SPEC.md, PRIVACY_SPEC.md

## Scope

Investigate suspected cross-tenant or cross-space memory access.

## Signals

- Customer report of foreign memory content
- Spike in `memory_authz_denied_total`
- Audit entries showing unexpected `spaceId` access

## Immediate containment

1. Enable production fail-closed auth (ensure `SDKWORK_IAM_DATABASE_URL` is set; disable `SDKWORK_MEMORY_DEV_AUTH_BYPASS`).
2. Scale deployment to zero if active exploitation is confirmed.
3. Preserve audit logs (`ai_audit_log`) and retrieval traces.

## Investigation

```powershell
pnpm verify
cargo test -p sdkwork-memory-integration-tests space_isolation
```

Review service-layer ACL in `crates/sdkwork-intelligence-memory-service/src/access.rs` and route manifest permissions.

## Recovery

1. Deploy patched release with verified isolation tests green.
2. Re-run tenant-scoped export/forget validation for affected tenants.
3. Document incident in `docs/changelogs/`.

## Escalation

Memory service owner → SDKWork security → customer success for tenant notification.
