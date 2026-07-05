# MEMORY Database Module

Canonical lifecycle assets for `sdkwork-memory` per `DATABASE_FRAMEWORK_SPEC.md`.

- moduleId: `memory`
- serviceCode: `MEMORY`
- tablePrefix: `ai_`
- engines: `postgres` (production), `sqlite` (local/dev standalone)

## Migration authority

Schema changes are authored in the native-sql plugin and materialized into the application-root baseline:

1. **Authoritative source** — `plugins/sdkwork-memory-plugin-native-sql/migrations/{postgres,sqlite}/V*.sql`
2. **Canonical baseline** — `database/ddl/baseline/{engine}/0001_memory_baseline.sql` (synced by `node tools/materialize_memory_migrations.mjs`)
3. **Post-GA migrations** — `database/migrations/{engine}/` is reserved for incremental schema changes after GA

Run `node tools/materialize_phase1_contracts.mjs` (or `pnpm api:materialize`) to refresh the baseline snapshot from plugin migrations before release verification.

## Initialization state

This module is in **initialization state** for greenfield deployments:

1. **Baseline** — `database/ddl/baseline/{engine}/0001_memory_baseline.sql` contains the full DDL snapshot.
2. **Migrations** — `database/migrations/{engine}/` is reserved for post-GA incremental schema changes only. It is intentionally empty at initialization.
3. **Drift** — run `pnpm db:drift:check` before release.

## Commands

```bash
pnpm run db:validate
pnpm run db:materialize:contract
pnpm run db:plan
pnpm run db:init
pnpm run db:migrate
pnpm run db:seed
pnpm run db:status
pnpm run db:drift:check
```
