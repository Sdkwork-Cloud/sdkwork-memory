# RUNBOOK: Migration Rollback

Status: active  
Owner: SDKWork Memory operators  
Specs: MIGRATION_SPEC.md, DATABASE_SPEC.md

## Scope

Recover from failed or incompatible database migrations for SDKWork Memory.

## Preconditions

- Database backup or point-in-time recovery available
- Known good release digest pinned in deployment manifest

## Forward migration path

Canonical migrations live in `database/migrations/{postgres,sqlite}/`.  
Materialize from plugin authority:

```powershell
node tools/materialize_memory_migrations.mjs
pnpm db:status
pnpm db:migrate
```

## Rollback

1. Stop API server pods (`kubectl scale deployment sdkwork-memory-standalone-gateway --replicas=0`).
2. Restore database from last verified backup, or apply paired `.down.sql` migrations in reverse order under `database/migrations/{postgres,sqlite}/`.
3. Redeploy previous container image digest.
4. Run smoke tests from `deployments/runbooks/rollout.md`.

## Verification

```powershell
pnpm db:drift:check
pnpm verify
```

## Escalation

Database platform on-call → Memory service owner.
