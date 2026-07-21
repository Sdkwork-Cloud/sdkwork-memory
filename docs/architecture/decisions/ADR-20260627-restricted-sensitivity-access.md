# ADR-20260627: Restricted Sensitivity Access Control

- Status: Accepted
- Date: 2026-06-27
- Deciders: Memory Platform Team

## Context

The `actor_may_read_sensitivity` function in `access.rs` used `elevated_tenant_access` as a blanket bypass for all sensitivity levels, including `restricted`. This meant any backend operator with elevated tenant access could read compliance-critical restricted data without additional authorization, violating the principle of least privilege.

## Decision

Modify `actor_may_read_sensitivity` to enforce tiered access:

1. **`public` / `internal`**: Always accessible.
2. **`private` / `sensitive`**: Accessible to the space owner or backend operators with `elevated_tenant_access` (with audit logging).
3. **`restricted`**: Requires explicit space ownership, even for backend operators with `elevated_tenant_access`. The elevated flag is denied for restricted data unless the actor is also the space owner.
4. **Unknown levels**: Default to the safest behavior (require space ownership).

## Consequences

- **Positive**: Compliance-critical data at the `restricted` level is protected from blanket admin access.
- **Positive**: Audit logging for elevated access to `private`/`sensitive` data provides an audit trail for security review.
- **Positive**: The principle of least privilege is enforced — elevated access is scoped, not absolute.
- **Negative**: Backend operators cannot perform bulk operations on `restricted` data without per-space ownership. This may require process changes for data migration or compliance audits.
- **Mitigation**: Future capability-based access control (via `MemoryCapabilityBinding`) can grant scoped `restricted` access to specific operators without blanket elevation.
