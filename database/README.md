# MEMORY Database Module

Canonical lifecycle assets for `sdkwork-memory` per `DATABASE_FRAMEWORK_SPEC.md`.

- moduleId: `memory`
- serviceCode: `MEMORY`
- tablePrefix: `mem_`

## Commands

```bash
pnpm run db:materialize:contract
pnpm run db:validate
```

Legacy SQL: `plugins/sdkwork-memory-plugin-native-sql/migrations/postgres/V202606100001__memory_phase1.sql` → `database/ddl/baseline/postgres/0001_memory_legacy_baseline.sql`

Runtime bootstrap: `sdkwork-memory-database-host` via `bootstrap_memory_database()` when postgres pool is configured; SQLite path continues to use `install_sqlite_schema()`.
