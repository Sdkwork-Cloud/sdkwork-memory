# sdkwork-memory
repository-kind: application

SDKWork Memory service and SDK families for embedding-optional AI memory, self-learning, habit memory, and provider-switchable retrieval.

## Standards

- Repository instructions: `AGENTS.md`
- Local component specs: `specs/README.md`
- Root SDKWork standards: `../sdkwork-specs/README.md`

## Verification

```powershell
pnpm verify
```

Canonical checks (also run individually):

```powershell
cargo test --workspace
pnpm check:pagination
pnpm check:api-envelope
pnpm db:validate
pnpm topology:validate
node tools/materialize_phase1_contracts.mjs
```

Postgres contract tests (optional, requires `SDKWORK_MEMORY_POSTGRES_TEST_URL`):

```powershell
$env:SDKWORK_MEMORY_POSTGRES_TEST_URL = "postgres://..."
cargo test -p sdkwork-memory-plugin-native-sql postgres_store_contract -- --nocapture
```

## Documentation Canon

- [docs/README.md](docs/README.md)
- [docs/product/prd/PRD.md](docs/product/prd/PRD.md)
- [docs/architecture/tech/TECH_ARCHITECTURE.md](docs/architecture/tech/TECH_ARCHITECTURE.md)

## Application Roots

- [apps directory index](apps/README.md)
