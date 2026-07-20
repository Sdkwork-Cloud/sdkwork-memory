# SDKWork Memory Technical Architecture

Status: active
Owner: SDKWork maintainers
Updated: 2026-07-20
Specs: ARCHITECTURE_DECISION_SPEC.md, DOCUMENTATION_SPEC.md

**Production readiness:** see `TECH-2026-06-10-commercial-memory-management-design.md` §2 for the authoritative current-state checklist (auth, pagination, FTS, jobs, deployment bootstrap).

## Document Map

- [TECH-2026-06-10-ai-memory-architecture-design.md](TECH-2026-06-10-ai-memory-architecture-design.md)
- [TECH-2026-06-10-commercial-memory-management-design.md](TECH-2026-06-10-commercial-memory-management-design.md)
- [TECH-2026-06-10-memory-implementation-family-baseline.md](TECH-2026-06-10-memory-implementation-family-baseline.md)
- [TECH-2026-06-10-memory-open-api-and-no-embedding-mvp.md](TECH-2026-06-10-memory-open-api-and-no-embedding-mvp.md)
- [TECH-2026-06-10-memory-spi-plugin-architecture-design.md](TECH-2026-06-10-memory-spi-plugin-architecture-design.md)
- [TECH-2026-06-10-memory-spi-plugin-runtime-implementation-plan.md](TECH-2026-06-10-memory-spi-plugin-runtime-implementation-plan.md)
- [TECH-2026-07-20-memory-commercial-retrieval-hardening.md](TECH-2026-07-20-memory-commercial-retrieval-hardening.md)
- [TECH-topology-standard.md](TECH-topology-standard.md)

## 1. Architecture Overview

SDKWork Memory is a Rust-based multi-tenant memory service organized as a Cargo workspace.
The service exposes three API surfaces (open / app / backend) through route-owned adapters
assembled by the `standalone-gateway` binary. Public deployment topology is expressed only by
the `standalone` or `cloud` deployment profile; process layout remains an internal orchestration detail.

The architecture follows a ports-and-adapters (hexagonal) pattern:

- **Contract crate** defines DTOs, ports (traits), and typed errors.
- **Service crate** implements business rules and depends on port traits, not concrete
  implementations.
- **Repository crate** implements the SQLx storage adapter against PostgreSQL and SQLite.
- **Plugin crates** implement SPI ports (native SQL, reference profiles).
- **Route crates** are thin HTTP handlers that delegate to service ports.
- **API server crate** assembles the HTTP runtime via `sdkwork-web-bootstrap` and injects
  concrete adapters into services.

Data flows: HTTP request → route handler → service port → repository/plugin adapter → database.

## 2. Technology Choices

| Choice | Selection | Rationale |
|---|---|---|
| Language | Rust (edition 2021) | Memory safety, zero-cost abstractions, async ecosystem |
| Web framework | axum + sdkwork-web-bootstrap | Type-safe routing, tower middleware, workspace standard |
| Database | PostgreSQL 15+ (production), SQLite (development) | ACID, JSONB, FTS, GIN indexes; SQLite for local dev |
| ORM | SQLx (compile-time checked) | Async, type-safe queries, dual-engine support |
| Migration | `database/migrations/{postgres,sqlite}/` with paired up/down | Framework-compliant, rollback-capable |
| Observability | tracing + tracing-subscriber + Prometheus | Structured logs, OpenTelemetry, metrics standard |
| Serialization | serde + serde_json | Rust ecosystem standard |
| ID generation | Snowflake (database-backed node registry) | Distributed-safe, sortable IDs |
| Auth | Dual-token (auth_token JWT + access_token JWT) | IAM-issued, tenant-scoped |
| SDK generation | sdkwork-sdk-generator (TypeScript) | OpenAPI-first, owner-only-input |

## 3. System Boundaries And Modules

### Crate Inventory

| Crate | Role | Depends On |
|---|---|---|
| `sdkwork-memory-contract` | DTOs, ports (traits), typed errors | none (domain) |
| `sdkwork-intelligence-memory-service` | Business rules, use case orchestration | contract, spi |
| `sdkwork-intelligence-memory-repository-sqlx` | SQLx adapter, bootstrap | contract, service, plugin-native-sql |
| `sdkwork-api-memory-standalone-gateway` | Binary entry point, HTTP bootstrap | all route crates, service, repository |
| `sdkwork-memory-spi` | SPI registry, ports, manifest validation | contract |
| `sdkwork-memory-retrieval` | Context pack fusion, retrieval algorithms | contract |
| `sdkwork-memory-profile-resolver` | Implementation profile resolution | contract, spi |
| `sdkwork-routes-memory-open-api` | Open API route handlers | contract, routes-memory-support |
| `sdkwork-routes-memory-app-api` | App API route handlers | contract, routes-memory-support |
| `sdkwork-routes-memory-backend-api` | Backend API route handlers | contract, routes-memory-support |
| `sdkwork-routes-memory-support` | Shared route support (auth, metrics, problem) | sdkwork-web-* framework |
| `sdkwork-memory-plugin-native-sql` | Native SQL storage plugin | contract, spi, repository-sqlx |
| `sdkwork-memory-plugin-reference-profiles` | Reference implementation profiles | contract, spi |
| `sdkwork-memory-integration-tests` | End-to-end test suite | all crates |

## 4. Directory And Package Layout

```
sdkwork-memory/
├── crates/
│   ├── sdkwork-memory-contract/           # DTOs, ports, errors
│   ├── sdkwork-intelligence-memory-service/        # Business rules
│   ├── sdkwork-intelligence-memory-repository-sqlx/ # SQLx adapter
│   ├── sdkwork-api-memory-standalone-gateway/         # Binary entry point
│   ├── sdkwork-memory-spi/                # SPI registry and ports
│   ├── sdkwork-memory-retrieval/               # Retrieval
│   ├── sdkwork-memory-profile-resolver/            # Profile resolver
│   ├── sdkwork-routes-memory-open-api/    # Open API routes
│   ├── sdkwork-routes-memory-app-api/     # App API routes
│   ├── sdkwork-routes-memory-backend-api/ # Backend API routes
│   ├── sdkwork-routes-memory-support/      # Shared route support
│   └── sdkwork-memory-integration-tests/  # E2E tests
├── plugins/
│   ├── sdkwork-memory-plugin-native-sql/  # Native SQL storage plugin
│   └── sdkwork-memory-plugin-reference-profiles/ # Reference profiles
├── apis/
│   ├── open-api/memory-open-api.openapi.json
│   ├── app-api/memory-app-api.openapi.json
│   ├── backend-api/memory-backend-api.openapi.json
│   ├── authority-manifest.json
│   └── rpc/README.md
├── sdks/
│   ├── sdkwork-memory-app-sdk/            # App API TypeScript SDK
│   ├── sdkwork-memory-backend-sdk/        # Backend API TypeScript SDK
│   ├── sdkwork-memory-sdk/                # Open API TypeScript SDK
│   └── _route-manifests/                  # Generated route manifests
├── database/
│   ├── migrations/{postgres,sqlite}/      # Paired up/down migrations
│   ├── contract/                          # prefix-registry, table-registry, schema.yaml
│   ├── drift/policy.yaml                  # Drift detection policy
│   ├── ddl/baseline/                      # Legacy baselines
│   └── seeds/                             # Seed manifests
├── deployments/
│   ├── docker/Dockerfile
│   ├── kubernetes/                        # Deployment, Service, HPA, PDB, Ingress, etc.
│   └── runbooks/rollout.md
├── etc/
│   ├── topology/                          # Four standard topology profiles
│   ├── sdkwork.deployment.config.json     # Source deployment index
│   └── sdkwork-api-cloud-gateway.memory.*.toml
├── docs/                                  # Product, architecture, engineering docs
└── tools/                                 # Contract materialization, verification scripts
```

## 5. API, SDK, And Data Ownership

### API Surfaces

| Surface | Prefix | Auth Mode | Operations |
|---|---|---|---|
| Open API | `/mem/v3/api` | ApiKey (X-API-Key) | 17 public operations |
| App API | `/app/v3/api/memory` | Dual-token (auth_token + access_token) | 33 end-user operations |
| Backend API | `/backend/v3/api/memory` | Dual-token (auth_token + access_token) | 41 admin operations |

### SDK Families

| SDK | Surface | Language | Location |
|---|---|---|---|
| `sdkwork-memory-sdk` | Open | TypeScript | `sdks/sdkwork-memory-sdk/` |
| `sdkwork-memory-app-sdk` | App | TypeScript | `sdks/sdkwork-memory-app-sdk/` |
| `sdkwork-memory-backend-sdk` | Backend | TypeScript | `sdks/sdkwork-memory-backend-sdk/` |

### Data Ownership

- **Memory service** owns: `ai_space`, `ai_event`, `ai_record`, `ai_record_source`,
  `ai_candidate`, `ai_habit`, `ai_retrieval_trace`, `ai_retrieval_hit`,
  `ai_context_pack`, `ai_index`, `ai_retrieval_profile`, `ai_implementation_profile`,
  `ai_provider_binding`, `ai_eval_run`, `ai_audit_log`, `ai_outbox_event`,
  `ai_tenant_preference`, `ai_learning_job`, `ai_record_fts` (sqlite virtual table),
  `ai_entity`, `ai_edge`, `ai_policy`, `ai_subject`, `ai_memory_binding`,
  `ai_capability_binding`, `ai_policy_assignment`, `ai_relation_rebuild_job`,
  `ai_commercial_readiness_snapshot`.
- **IAM service** owns: identity, tenant, organization, access token tables.
- All memory tables use the `ai_` prefix (per `database/contract/prefix-registry.json`).

## 6. Security, Privacy, And Observability

### Security

- **Authentication**: Dual-token (auth_token JWT + access_token JWT) for app/backend;
  ApiKey for open API.
- **Authorization**: Fail-closed access control via `access.rs` — every read/write checks
  space ownership and sensitivity level.
- **Tenant isolation**: Every table has `tenant_id NOT NULL`; every index has `tenant_id`
  as the leading column.
- **Dev auth bypass**: `SDKWORK_MEMORY_DEV_AUTH_BYPASS=true` is development-only and MUST
  be false in production. The bootstrap loader rejects `DEV_*` variables in production.
- **Secrets**: Provider binding `secret_ref` stores a reference pointer, never the secret
  itself. Secrets are managed by the approved secret manager/KMS.

### Privacy

- **Sensitivity classification**: `public / internal / private / sensitive / restricted`.
- **Soft delete**: `deleted_at` + `status='deleted'` for recoverability.
- **Hard delete**: `forget_*` repository methods physically delete records and derivatives.
- **PII detection**: `sensitive_content.rs` detects credential-like patterns (extending to
  email/phone/IP in Phase 2).
- **Export**: Export jobs produce `exportRef` (not inline payload) with sensitivity filtering.
- **Audit**: All mutations write to `ai_audit_log` with action, resource, result, tenant.

### Observability

- **Metrics**: Prometheus `/metrics` endpoint with HTTP and domain metrics.
- **Tracing**: `tracing-subscriber` with OpenTelemetry OTLP export (otel feature).
- **Readiness**: `/readyz` checks database + IAM connectivity; `/healthz` for liveness.
- **Correlation**: `correlation.rs` injects `request_id`/`trace_id` into all spans.
- **Audit log**: Separate `ai_audit_log` table for compliance-grade event recording.
- **Qualified scheme labels**: runtime metrics distinguish the bounded
  PostgreSQL/SQLite and balanced/search-first/event-aware combinations.
- **Offline quality evaluation**: bounded golden cases execute the production
  retrieval path and persist Recall@K, Hit Rate@K, MRR, degraded rate, and
  explicit quality-gate outcomes; absent datasets fail closed.
- **Query identity**: normalized SHA-256 query hashes support deterministic
  trace/eval correlation without using implementation-defined hash output.
- **Consolidation**: exact canonical duplicates are transactionally linked by
  supersession within user, scope, type, and sensitivity boundaries.

## 7. Deployment And Runtime Topology

### Topology Matrix

| Topology | Profile | Layout | Use Case |
|---|---|---|---|
| `standalone.development` | standalone | Local source runtime | Local development, SQLite by default |
| `standalone.production` | standalone | Container or binary | Single-site production, PostgreSQL |
| `cloud.development` | cloud | Platform-managed | Shared development environment |
| `cloud.production` | cloud | Kubernetes | Production, PostgreSQL |

### Runtime Targets

- `container`: Docker / Kubernetes deployment.
- `binary`: Bare metal / VM deployment.

### Database

- **Development**: SQLite in-memory (`sqlite::memory:`) for quick iteration.
- **Production**: PostgreSQL 15+ (via `SDKWORK_MEMORY_DATABASE_URL` secret).
- **Migration**: Explicit `db-migrate` command or migration Job; `autoMigrate` is false in
  production.

## 8. Architecture Decision Index

> ADR-* records will be created in `docs/architecture/decisions/` as decisions are formalized.

| Decision | Status | ADR |
|---|---|---|
| Embedding-optional retrieval (FTS-first) | Accepted | Pending ADR-001 |
| SPI plugin architecture for provider switching | Accepted | Pending ADR-002 |
| Table prefix `ai_` (not `mem_`) | Accepted | Pending ADR-003 |
| Dual-engine support (PostgreSQL + SQLite) | Accepted | Pending ADR-004 |
| Three API surfaces (open / app / backend) | Accepted | Pending ADR-005 |
| Snowflake ID with database-backed node registry | Accepted | Pending ADR-006 |
| Outbox pattern for event publishing | Accepted | Pending ADR-007 |
| Soft delete + hard delete for privacy compliance | Accepted | Pending ADR-008 |

## 9. Verification

### Build

```bash
cargo check --workspace
cargo clippy --workspace -- -D warnings
cargo test --workspace
```

### Contract Verification

```bash
node tools/materialize_phase1_contracts.mjs
powershell -ExecutionPolicy Bypass -File tools/verify_phase1.ps1
```

### Migration Verification

```bash
# Apply migrations to a clean PostgreSQL database
cargo run -p sdkwork-api-memory-standalone-gateway -- db-migrate

# Verify rollback
# (Apply down migrations in reverse order via the migration runner)
```

### Deployment Verification

```bash
# Docker build
docker build -f deployments/docker/Dockerfile -t sdkwork-api-memory-standalone-gateway:dev .

# Kubernetes dry-run
kubectl apply --dry-run=client -f deployments/kubernetes/
```
